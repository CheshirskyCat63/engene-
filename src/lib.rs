//! ENGENE root package = thin migration shell.
//! Public surface is intentionally narrow.
//! Internal legacy/domain modules remain mounted privately until ownership handoff completes.
pub mod app;
pub mod core;
pub mod runtime;
pub mod testsupport;
mod audio;
mod body;
mod game;
mod graphics;
mod input;
mod memory;
mod navigation;
mod network;
mod simulation;
mod tools;
mod world;
