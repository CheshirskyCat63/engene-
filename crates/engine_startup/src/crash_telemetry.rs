use std::sync::Once;
use std::panic;
use std::fs;
use std::path::Path;
use chrono::Utc;

static INIT_CRASH_HANDLER: Once = Once::new();

pub fn crash_telemetry() {
    INIT_CRASH_HANDLER.call_once(|| {
        let original_hook = panic::take_hook();
        
        panic::set_hook(Box::new(move |panic_info| {
            let timestamp = Utc::now().format("%Y-%m-%d_%H-%M-%S").to_string();
            let crash_dir = "game/crashes";
            
            if let Err(e) = fs::create_dir_all(crash_dir) {
                eprintln!("Failed to create crash directory: {}", e);
                return;
            }
            
            let crash_file = format!("{}/crash_{}.log", crash_dir, timestamp);
            
            let panic_msg = match panic_info.payload().downcast_ref::<String>() {
                Some(s) => s.clone(),
                None => match panic_info.payload().downcast_ref::<&str>() {
                    Some(s) => s.to_string(),
                    None => "Unknown panic".to_string(),
                }
            };
            
            let location = match panic_info.location() {
                Some(loc) => format!("{}:{}", loc.file(), loc.line()),
                None => "Unknown location".to_string(),
            };
            
            let crash_report = format!(
                "=== CRASH REPORT ===\n\
                 Timestamp: {}\n\
                 Panic Message: {}\n\
                 Location: {}\n\
                 Stack Trace:\n\
                 {:#?}\n\
                 ===================",
                timestamp,
                panic_msg,
                location,
                backtrace::Backtrace::new()
            );
            
            if let Err(e) = fs::write(&crash_file, &crash_report) {
                eprintln!("Failed to write crash report: {}", e);
            } else {
                eprintln!("Crash report written to: {}", crash_file);
            }
            
            original_hook(panic_info);
        }));
    });
}

pub fn install_panic_hook() {
    crash_telemetry();
}
