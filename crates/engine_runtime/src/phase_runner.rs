//! Canonical Phase Runner - Unified phase execution engine.
//!
//! This module provides the complete phase execution system that runs
//! all canonical phases in proper order with proper error handling
//! and state management.

use std::sync::Arc;
use crate::phase::{
    Phase, PhaseContext, PhaseResult, PhaseTrait,
    validate_phase_order, run_all_phases,
};
use crate::simulation_core::systems::{EngineSystem, FixedTickContext, WorldTickSystem};

/// Complete phase execution state
#[derive(Debug)]
pub struct PhaseRunner {
    phases: Vec<Box<dyn PhaseTrait + Send + Sync>>,
    context: PhaseContext,
}

impl PhaseRunner {
    /// Create a new phase runner with all canonical phases.
    pub fn new() -> Self {
        let phases: Vec<Box<dyn PhaseTrait + Send + Sync>> = vec![
            Box::new(crate::phase::tick::TickPhase),
            Box::new(crate::phase::streaming::StreamingPhase),
            Box::new(crate::phase::persistence::PersistencePhase),
            Box::new(crate::phase::spatial::SpatialPhase),
            Box::new(crate::phase::audio::AudioPhase),
            Box::new(crate::phase::editor::EditorPhase),
            Box::new(crate::phase::render::RenderPhase),
        ];
        
        Self {
            phases,
            context: PhaseContext {
                tick: 0,
                delta_seconds: 0.0,
                is_editor_mode: false,
            },
        }
    }
    
    /// Execute all phases in canonical order.
    pub fn execute_frame(&mut self, delta_seconds: f32) -> PhaseResult {
        self.context.delta_seconds = delta_seconds;
        self.context.tick += 1;
        
        // Run all phases in order
        let results = run_all_phases(&self.phases, &self.context);
        
        // Find first error if any
        for result in &results {
            if !result.success {
                return result.clone();
            }
        }
        
        // All phases succeeded
        let total_duration: f32 = results.iter().map(|r| r.duration_ms).sum();
        
        PhaseResult {
            success: true,
            duration_ms: total_duration,
            error_message: None,
        }
    }
    
    /// Execute phases with custom phase list.
    pub fn execute_custom_phases(
        &mut self,
        phases: &[Box<dyn PhaseTrait + Send + Sync>]
    ) -> PhaseResult {
        let results = run_all_phases(phases, &self.context);
        
        for result in &results {
            if !result.success {
                return result.clone();
            }
        }
        
        let total_duration: f32 = results.iter().map(|r| r.duration_ms).sum();
        
        PhaseResult {
            success: true,
            duration_ms: total_duration,
            error_message: None,
        }
    }
    
    /// Get current context for external systems.
    pub fn context(&self) -> &PhaseContext {
        &self.context
    }
    
    /// Get current tick number.
    pub fn tick(&self) -> u64 {
        self.context.tick
    }
    
    /// Check if runner is in editor mode.
    pub fn is_editor_mode(&self) -> bool {
        self.context.is_editor_mode
    }
    
    /// Set editor mode.
    pub fn set_editor_mode(&mut self, is_editor: bool) {
        self.context.is_editor_mode = is_editor;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::phase::tick::TickPhase;
    use crate::phase::streaming::StreamingPhase;
    
    #[test]
    fn test_phase_runner_creation() {
        let runner = PhaseRunner::new();
        assert_eq!(runner.phases.len(), 7); // All canonical phases
        
        // Verify phase order
        let expected_order = vec![
            Phase::Tick,
            Phase::Streaming,
            Phase::Persistence,
            Phase::Spatial,
            Phase::Audio,
            Phase::EditorUpdate,
            Phase::Render,
        ];
        
        for (i, phase) in runner.phases.iter().enumerate() {
            assert_eq!(phase.phase_type(), expected_order[i]);
        }
    }
    
    #[test]
    fn test_phase_runner_execution() {
        let mut runner = PhaseRunner::new();
        
        // Mock successful execution
        let result = runner.execute_frame(0.016); // ~60 FPS
        
        assert!(result.success);
        assert!(result.error_message.is_none());
        assert!(result.duration_ms > 0.0);
        
        // Verify tick advanced
        assert_eq!(runner.tick(), 1);
    }
    
    #[test]
    fn test_phase_runner_error_handling() {
        let mut runner = PhaseRunner::new();
        
        // Create a failing phase
        struct FailingPhase;
        impl PhaseTrait for FailingPhase {
            fn execute(&self, _ctx: &PhaseContext) -> PhaseResult {
                PhaseResult::error("Test failure", 0.0)
            }
            
            fn phase_type(&self) -> Phase { Phase::Tick }
        }
        
        // Replace tick phase with failing phase
        runner.phases[0] = Box::new(FailingPhase);
        
        let result = runner.execute_frame(0.016);
        
        // Should fail
        assert!(!result.success);
        assert!(result.error_message.is_some());
    }
}
