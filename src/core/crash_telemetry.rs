//! Crash Telemetry Pipeline — captures panic context and writes crash bundles.
//!
//! On panic: captures stack trace, timestamp, and basic system info.
//! Writes crash bundle to `crashes/crash_{timestamp}.bin`.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Mutex;

static CRASH_CONTEXT: Mutex<CrashContext> = Mutex::new(CrashContext::empty());

pub struct CrashContext {
    pub entity_count: usize,
    pub active_system: Option<&'static str>,
    pub recent_events: Vec<String>,
    pub loaded_chunks: Vec<String>,
}

impl CrashContext {
    const fn empty() -> Self {
        Self {
            entity_count: 0,
            active_system: None,
            recent_events: Vec::new(),
            loaded_chunks: Vec::new(),
        }
    }

    pub fn record_event(description: String) {
        if let Ok(mut ctx) = CRASH_CONTEXT.lock() {
            ctx.recent_events.push(description);
            if ctx.recent_events.len() > 100 {
                ctx.recent_events.remove(0);
            }
        }
    }

    pub fn set_active_system(name: &'static str) {
        if let Ok(mut ctx) = CRASH_CONTEXT.lock() {
            ctx.active_system = Some(name);
        }
    }

    pub fn clear_active_system() {
        if let Ok(mut ctx) = CRASH_CONTEXT.lock() {
            ctx.active_system = None;
        }
    }

    pub fn set_entity_count(count: usize) {
        if let Ok(mut ctx) = CRASH_CONTEXT.lock() {
            ctx.entity_count = count;
        }
    }

    pub fn set_loaded_chunks(chunks: Vec<String>) {
        if let Ok(mut ctx) = CRASH_CONTEXT.lock() {
            ctx.loaded_chunks = chunks;
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrashBundle {
    pub timestamp: String,
    pub panic_message: String,
    pub location: Option<String>,
    pub entity_count: Option<usize>,
    pub active_system: Option<String>,
    pub engine_version: String,
    pub profile: String,
    pub recent_events: Vec<String>,
    pub loaded_chunks: Vec<String>,
}

impl CrashBundle {
    pub fn crash_dir() -> PathBuf {
        PathBuf::from("crashes")
    }

    pub fn save(&self) -> Result<PathBuf, std::io::Error> {
        let dir = Self::crash_dir();
        std::fs::create_dir_all(&dir)?;

        let safe_ts = self.timestamp.replace([':', ' '], "_");
        let filename = format!("crash_{}.ron", safe_ts);
        let path = dir.join(&filename);

        let serialized = ron::ser::to_string_pretty(self, ron::ser::PrettyConfig::default())
            .unwrap_or_else(|_| format!("{:?}", self));
        std::fs::write(&path, serialized)?;

        Ok(path)
    }

    pub fn list_recent(max_count: usize) -> Vec<PathBuf> {
        let dir = Self::crash_dir();
        if !dir.exists() {
            return Vec::new();
        }
        let mut entries: Vec<_> = std::fs::read_dir(&dir)
            .into_iter()
            .flatten()
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path()
                    .extension()
                    .map_or(false, |ext| ext == "ron" || ext == "bin")
            })
            .collect();

        entries.sort_by(|a, b| {
            b.metadata()
                .and_then(|m| m.modified())
                .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
                .cmp(
                    &a.metadata()
                        .and_then(|m| m.modified())
                        .unwrap_or(std::time::SystemTime::UNIX_EPOCH),
                )
        });

        entries
            .into_iter()
            .take(max_count)
            .map(|e| e.path())
            .collect()
    }
}

/// Installs a panic hook that captures crash context and writes a crash bundle.
pub fn install_panic_hook() {
    let default_hook = std::panic::take_hook();

    std::panic::set_hook(Box::new(move |info| {
        let message = if let Some(s) = info.payload().downcast_ref::<&str>() {
            s.to_string()
        } else if let Some(s) = info.payload().downcast_ref::<String>() {
            s.clone()
        } else {
            "unknown panic".to_string()
        };

        let location = info.location().map(|loc| {
            format!("{}:{}:{}", loc.file(), loc.line(), loc.column())
        });

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default();
        let timestamp = format!("{}", now.as_secs());

        let (entity_count, active_system, recent_events, loaded_chunks) =
            if let Ok(ctx) = CRASH_CONTEXT.lock() {
                (
                    Some(ctx.entity_count),
                    ctx.active_system.map(|s| s.to_string()),
                    ctx.recent_events.clone(),
                    ctx.loaded_chunks.clone(),
                )
            } else {
                (None, None, Vec::new(), Vec::new())
            };

        let bundle = CrashBundle {
            timestamp,
            panic_message: message,
            location,
            entity_count,
            active_system,
            engine_version: env!("CARGO_PKG_VERSION").to_string(),
            profile: if cfg!(debug_assertions) {
                "dev".to_string()
            } else {
                "shipping".to_string()
            },
            recent_events,
            loaded_chunks,
        };

        match bundle.save() {
            Ok(path) => eprintln!("[crash_telemetry] crash bundle saved to {:?}", path),
            Err(e) => eprintln!("[crash_telemetry] failed to save crash bundle: {}", e),
        }

        default_hook(info);
    }));
}
