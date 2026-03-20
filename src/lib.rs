//! ENGENE root package = thin migration shell.
//! Public surface is intentionally narrow.
//! Internal legacy/domain modules remain mounted privately until ownership handoff completes.
pub mod app;
pub mod core;
pub mod runtime;
pub mod testsupport;
mod animation;
mod audio;
mod body;
mod content;
mod game;
mod graphics;
mod input;
mod memory;
mod navigation;
mod network;
mod physics;
mod simulation;
mod tools;
mod world;
