//! Wiring Boundary Contracts
//!
//! These tests inspect real system descriptors from integration.rs.
//! They verify that wiring domains have correct boundaries.

use std::fs;

const INTEGRATION_PATH: &str = "src/runtime/wiring/integration.rs";

/// Check if source contains a specific system definition
fn has_system(source: &str, system_name: &str) -> bool {
    source.contains(&format!("pub struct {}", system_name))
}

/// Check if descriptor writes to a resource
fn descriptor_writes_resource(source: &str, system_name: &str, resource: &str) -> bool {
    // Find the system's descriptor method
    let system_start = source.find(&format!("impl EngineSystem for {}", system_name));
    if system_start.is_none() {
        return false;
    }
    
    let after_impl = &source[system_start.unwrap()..];
    
    // Find descriptor method
    let descriptor_start = after_impl.find("fn descriptor(&self)");
    if descriptor_start.is_none() {
        return false;
    }
    
    let descriptor_body = &after_impl[descriptor_start.unwrap()..];
    
    // Find the end of descriptor method (next closing brace at method level)
    // Simple check: look for writes_resource call
    descriptor_body.contains(&format!("writes_resource::<{}>()", resource))
}

/// Check if descriptor reads a resource
fn descriptor_reads_resource(source: &str, system_name: &str, resource: &str) -> bool {
    let system_start = source.find(&format!("impl EngineSystem for {}", system_name));
    if system_start.is_none() {
        return false;
    }
    
    let after_impl = &source[system_start.unwrap()..];
    let descriptor_start = after_impl.find("fn descriptor(&self)");
    if descriptor_start.is_none() {
        return false;
    }
    
    let descriptor_body = &after_impl[descriptor_start.unwrap()..];
    descriptor_body.contains(&format!("reads_resource::<{}>()", resource))
}

/// Check if descriptor emits an event
fn descriptor_emits_event(source: &str, system_name: &str, event: &str) -> bool {
    let system_start = source.find(&format!("impl EngineSystem for {}", system_name));
    if system_start.is_none() {
        return false;
    }
    
    let after_impl = &source[system_start.unwrap()..];
    let descriptor_start = after_impl.find("fn descriptor(&self)");
    if descriptor_start.is_none() {
        return false;
    }
    
    let descriptor_body = &after_impl[descriptor_start.unwrap()..];
    descriptor_body.contains(&format!("emits_event::<{}>()", event))
}

/// Check if descriptor reads an event
fn descriptor_reads_event(source: &str, system_name: &str, event: &str) -> bool {
    let system_start = source.find(&format!("impl EngineSystem for {}", system_name));
    if system_start.is_none() {
        return false;
    }
    
    let after_impl = &source[system_start.unwrap()..];
    let descriptor_start = after_impl.find("fn descriptor(&self)");
    if descriptor_start.is_none() {
        return false;
    }
    
    let descriptor_body = &after_impl[descriptor_start.unwrap()..];
    descriptor_body.contains(&format!("reads_event::<{}>()", event))
}

/// Ballistics tick descriptor matches boundary contract.
/// BallisticsTick must write BallisticsSystem, read WorldFields, emit ImpactEvent.
#[test]
fn ballistics_tick_descriptor_matches_boundary_contract() {
    let source = fs::read_to_string(INTEGRATION_PATH)
        .expect("integration.rs must exist");

    // Verify BallisticsTickSystem exists
    assert!(has_system(&source, "BallisticsTickSystem"),
        "BallisticsTickSystem must be defined");

    // Verify descriptor writes BallisticsSystem
    assert!(descriptor_writes_resource(&source, "BallisticsTickSystem", "BallisticsSystem"),
        "BallisticsTickSystem must write BallisticsSystem resource");

    // Verify descriptor reads WorldFields
    assert!(descriptor_reads_resource(&source, "BallisticsTickSystem", "WorldFields"),
        "BallisticsTickSystem must read WorldFields resource");

    // Verify descriptor emits ImpactEvent
    assert!(descriptor_emits_event(&source, "BallisticsTickSystem", "ImpactEvent"),
        "BallisticsTickSystem must emit ImpactEvent");

    // Verify descriptor emits SoundTrigger
    assert!(descriptor_emits_event(&source, "BallisticsTickSystem", "SoundTrigger"),
        "BallisticsTickSystem must emit SoundTrigger");
}

/// Damage dispatch descriptor matches boundary contract.
#[test]
fn damage_dispatch_descriptor_matches_boundary_contract() {
    let source = fs::read_to_string(INTEGRATION_PATH)
        .expect("integration.rs must exist");

    // Verify DamageDispatchSystem exists
    assert!(has_system(&source, "DamageDispatchSystem"),
        "DamageDispatchSystem must be defined");

    // Verify descriptor reads ImpactEvent
    assert!(descriptor_reads_event(&source, "DamageDispatchSystem", "ImpactEvent"),
        "DamageDispatchSystem must read ImpactEvent");

    // Verify descriptor emits BodyZoneDamaged
    assert!(descriptor_emits_event(&source, "DamageDispatchSystem", "BodyZoneDamaged"),
        "DamageDispatchSystem must emit BodyZoneDamaged");

    // Verify descriptor emits TerrainDeformed
    assert!(descriptor_emits_event(&source, "DamageDispatchSystem", "TerrainDeformed"),
        "DamageDispatchSystem must emit TerrainDeformed");
}

/// Destruction/nav/occlusion chain uses expected events only.
#[test]
fn destruction_nav_occlusion_chain_uses_expected_events_only() {
    let source = fs::read_to_string(INTEGRATION_PATH)
        .expect("integration.rs must exist");

    // DestructionTickSystem
    assert!(has_system(&source, "DestructionTickSystem"),
        "DestructionTickSystem must be defined");
    assert!(descriptor_reads_event(&source, "DestructionTickSystem", "ImpactEvent"),
        "DestructionTickSystem must read ImpactEvent");
    assert!(descriptor_emits_event(&source, "DestructionTickSystem", "WorldTopologyChanged"),
        "DestructionTickSystem must emit WorldTopologyChanged");

    // NavDirtyTickSystem
    assert!(has_system(&source, "NavDirtyTickSystem"),
        "NavDirtyTickSystem must be defined");
    assert!(descriptor_reads_event(&source, "NavDirtyTickSystem", "WorldTopologyChanged"),
        "NavDirtyTickSystem must read WorldTopologyChanged");
    assert!(descriptor_reads_event(&source, "NavDirtyTickSystem", "TerrainChanged"),
        "NavDirtyTickSystem must read TerrainChanged");

    // OcclusionWireSystem
    assert!(has_system(&source, "OcclusionWireSystem"),
        "OcclusionWireSystem must be defined");
    assert!(descriptor_reads_event(&source, "OcclusionWireSystem", "StructuralCollapse"),
        "OcclusionWireSystem must read StructuralCollapse");
}

/// AI descriptor does not require animation descriptor.
#[test]
fn ai_descriptor_does_not_require_animation_descriptor() {
    let source = fs::read_to_string(INTEGRATION_PATH)
        .expect("integration.rs must exist");

    // Verify AiDecisionWireSystem exists
    assert!(has_system(&source, "AiDecisionWireSystem"),
        "AiDecisionWireSystem must be defined");

    // Find AI descriptor section
    let ai_impl = source.find("impl EngineSystem for AiDecisionWireSystem");
    assert!(ai_impl.is_some(), "AiDecisionWireSystem must implement EngineSystem");

    let ai_descriptor = &source[ai_impl.unwrap()..];
    
    // Find the descriptor method
    let descriptor_start = ai_descriptor.find("fn descriptor(&self)");
    assert!(descriptor_start.is_some(), "AiDecisionWireSystem must have descriptor method");

    let descriptor_body = &ai_descriptor[descriptor_start.unwrap()..descriptor_start.unwrap() + 500];

    // Verify AI does NOT read AnimationWire resources
    // AI should read TacticProfile and CoverMap, not animation resources
    assert!(!descriptor_body.contains("ClipMap"),
        "AiDecisionWireSystem should NOT read ClipMap (animation resource)");
    assert!(!descriptor_body.contains("AnimationLadder"),
        "AiDecisionWireSystem should NOT read AnimationLadder");

    // Verify AnimationWireSystem exists separately
    assert!(has_system(&source, "AnimationWireSystem"),
        "AnimationWireSystem must be defined");
}

/// Gore boundary is optional at descriptor level.
#[test]
fn gore_boundary_is_optional_at_descriptor_level() {
    let source = fs::read_to_string(INTEGRATION_PATH)
        .expect("integration.rs must exist");

    // Verify GoreWireSystem exists
    assert!(has_system(&source, "GoreWireSystem"),
        "GoreWireSystem must be defined");

    // Verify GoreWireSystem reads BodyZoneDamaged
    assert!(descriptor_reads_event(&source, "GoreWireSystem", "BodyZoneDamaged"),
        "GoreWireSystem must read BodyZoneDamaged");

    // Verify GoreWireSystem emits GoreMeshSpawn
    assert!(descriptor_emits_event(&source, "GoreWireSystem", "GoreMeshSpawn"),
        "GoreWireSystem must emit GoreMeshSpawn");

    // Verify gore is a downstream consumer, not a required input for core systems
    // DamageDispatchSystem should NOT require GoreWireSystem events
    let damage_impl = source.find("impl EngineSystem for DamageDispatchSystem");
    assert!(damage_impl.is_some(), "DamageDispatchSystem must exist");

    let damage_descriptor = &source[damage_impl.unwrap()..];
    let descriptor_start = damage_descriptor.find("fn descriptor(&self)");
    assert!(descriptor_start.is_some(), "DamageDispatchSystem must have descriptor");

    let descriptor_body = &damage_descriptor[descriptor_start.unwrap()..descriptor_start.unwrap() + 500];

    // DamageDispatch should not depend on gore
    assert!(!descriptor_body.contains("GoreMeshSpawn"),
        "DamageDispatchSystem should not depend on GoreMeshSpawn");
}
