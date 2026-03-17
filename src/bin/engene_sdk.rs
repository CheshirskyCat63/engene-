//! ENGENE SDK binary bootstrap entrypoint.

use engene::app::sdk_runner::{run, SdkStartupMode};

fn parse_startup_mode() -> SdkStartupMode {
    let args: Vec<String> = std::env::args().collect();
    for i in 0..args.len() {
        if args[i] == "--mode" {
            if let Some(mode) = args.get(i + 1) {
                if mode.eq_ignore_ascii_case("editor") {
                    return SdkStartupMode::Editor;
                }
            }
        }
    }
    SdkStartupMode::Editor
}

fn main() {
    run(parse_startup_mode());
}
