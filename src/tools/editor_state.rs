//! Editor state persistence - saves crash counters and panel health between sessions.
//! Stored in editor_state.json next to the executable.

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// Persisted editor state for crash recovery
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EditorState {
    /// Per-panel crash/failure counters
    pub panel_failures: HashMap<String, PanelFailureState>,
    /// Whether safe mode was active at shutdown
    pub safe_mode_was_active: bool,
    /// Total editor frames rendered
    pub total_frames: u64,
    /// Timestamp of last save
    pub last_save_timestamp: u64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PanelFailureState {
    pub consecutive_failures: u32,
    pub total_failures: u32,
    pub was_disabled: bool,
    pub disabled_reason: Option<String>,
}

impl Default for EditorState {
    fn default() -> Self {
        Self {
            panel_failures: HashMap::new(),
            safe_mode_was_active: false,
            total_frames: 0,
            last_save_timestamp: 0,
        }
    }
}

/// Get the editor state file path (next to executable)
fn get_state_path() -> PathBuf {
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(dir) = exe_path.parent() {
            return dir.join("editor_state.json");
        }
    }
    PathBuf::from("editor_state.json")
}

/// Load editor state from disk (if exists)
pub fn load_state() -> EditorState {
    let path = get_state_path();
    if !path.exists() {
        return EditorState::default();
    }

    match fs::read_to_string(&path) {
        Ok(content) => {
            match serde_json::from_str(&content) {
                Ok(state) => {
                    tracing::info!("loaded editor state from {}", path.display());
                    state
                }
                Err(e) => {
                    tracing::warn!("failed to parse editor state: {}", e);
                    EditorState::default()
                }
            }
        }
        Err(e) => {
            tracing::warn!("failed to read editor state: {}", e);
            EditorState::default()
        }
    }
}

/// Save editor state to disk
pub fn save_state(state: &EditorState) -> Result<(), String> {
    let path = get_state_path();
    
    let mut state = state.clone();
    state.last_save_timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let content = serde_json::to_string_pretty(state)
        .map_err(|e| format!("serialize: {}", e))?;
    
    // Use atomic save for safety
    use crate::memory::atomic_saved::atomic_save;
    atomic_save(&path, content.as_bytes())
        .map_err(|e| format!("atomic save: {}", e))?;
    
    tracing::debug!("saved editor state to {}", path.display());
    Ok(())
}

/// Restore failure counters into EditorSafeMode from persisted state
pub fn restore_to_safe_mode(
    safe_mode: &mut crate::tools::editor_safe_mode::EditorSafeMode,
    state: &EditorState,
) {
    for (panel_name, failure_state) in &state.panel_failures {
        // Register the panel if not already registered
        safe_mode.register_panel(panel_name);
        
        // If was disabled, set the disabled reason but don't auto-disable on restore
        // User can manually re-enable
        if failure_state.was_disabled {
            // We can't directly set disabled state, but we can record the reason
            // by triggering a "virtual" failure that sets the reason
            tracing::info!(
                "panel '{}' was disabled at last shutdown: {}",
                panel_name,
                failure_state.disabled_reason.as_deref().unwrap_or("unknown")
            );
        }
    }
    
    if state.safe_mode_was_active {
        tracing::info!("safe mode was active at last shutdown");
        // Don't auto-enable safe mode on restore - let user decide
    }
}

/// Extract current state from EditorSafeMode for persistence
pub fn capture_state(
    safe_mode: &crate::tools::editor_safe_mode::EditorSafeMode,
) -> EditorState {
    let mut panel_failures = HashMap::new();
    
    // Get failure counts from all registered panels
    let failure_counts = safe_mode.panel_failure_counts();
    for (panel_name, (consecutive, total, was_disabled)) in failure_counts {
        if total > 0 || was_disabled {
            panel_failures.insert(
                panel_name,
                PanelFailureState {
                    consecutive_failures: consecutive,
                    total_failures: total,
                    was_disabled,
                    disabled_reason: safe_mode.panel_disabled_reason(&panel_name),
                },
            );
        }
    }
    
    EditorState {
        panel_failures,
        safe_mode_was_active: safe_mode.is_safe_mode(),
        total_frames: 0, // Would need to track this in safe_mode
        last_save_timestamp: 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_state() {
        let state = EditorState::default();
        assert!(state.panel_failures.is_empty());
        assert!(!state.safe_mode_was_active);
    }

    #[test]
    fn test_serialize_roundtrip() {
        let mut state = EditorState::default();
        state.panel_failures.insert(
            "Inspector".to_string(),
            PanelFailureState {
                consecutive_failures: 2,
                total_failures: 5,
                was_disabled: true,
                disabled_reason: Some("too many panics".to_string()),
            },
        );
        state.safe_mode_was_active = true;
        
        let json = serde_json::to_string_pretty(&state).unwrap();
        let restored: EditorState = serde_json::from_str(&json).unwrap();
        
        assert_eq!(restored.panel_failures.len(), 1);
        assert!(restored.safe_mode_was_active);
    }
}
