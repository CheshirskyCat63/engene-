//! ENGENE workspace root
//! 
//! This is a workspace-only package that doesn't contain any code.
//! All functionality has been moved to canonical crates.
//! 
//! Architecture:
//! - engine_core: Core policies and time
//! - engine_ecs: Entity component system
//! - engine_world: World systems and components
//! - engine_runtime: Runtime assembly and systems
//! - engine_render: Rendering pipeline
//! - engine_physics: Physics simulation
//! - engine_audio: Audio systems
//! - engine_content: Content management
//! - engine_tools: Development tools
//! - engine_startup: Bootstrap utilities
//! - sdk_app: SDK application framework
//! - game_framework: Game-specific framework

pub use engine_core;
pub use engine_ecs;
pub use engine_world;
pub use engine_runtime;
pub use engine_render;
pub use engine_physics;
pub use engine_audio;
pub use engine_content;
pub use engine_tools;
pub use engine_startup;
pub use sdk_app;
pub use game_framework;
