use std::env;
use std::process::{Command, ExitCode};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BootstrapProfile {
    Game,
    Sdk,
    Headless,
    Tools,
}

impl BootstrapProfile {
    fn parse(args: &[String]) -> Result<Self, String> {
        if args.iter().any(|a| a == "--sdk") {
            return Ok(Self::Sdk);
        }
        if args.iter().any(|a| a == "--headless") {
            return Ok(Self::Headless);
        }
        if args.iter().any(|a| a == "--tools") {
            return Ok(Self::Tools);
        }
        Ok(Self::Game)
    }

    fn target_exe(self) -> &'static str {
        match self {
            Self::Game => "engene_game",
            Self::Sdk => "engene_sdk",
            Self::Headless => "engene_headless",
            Self::Tools => "engene_tools",
        }
    }
}

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();

    let profile = match BootstrapProfile::parse(&args) {
        Ok(p) => p,
        Err(err) => {
            eprintln!("bootstrap profile parse error: {err}");
            return ExitCode::FAILURE;
        }
    };

    let child_args: Vec<_> = args.iter().skip(1).collect();

    println!("bootstrapping into '{}' with args: {:?}", profile.target_exe(), child_args);

    let status = Command::new(profile.target_exe())
        .args(child_args)
        .status();

    match status {
        Ok(status) if status.success() => ExitCode::SUCCESS,
        Ok(status) => {
            eprintln!("child process exited with status: {status}");
            ExitCode::FAILURE
        }
        Err(err) => {
            eprintln!("failed to start '{}': {err}", profile.target_exe());
            eprintln!("make sure the target executable is in your PATH, or run this via 'cargo run -p engene_bootstrap -- <args>'");
            ExitCode::FAILURE
        }
    }
}
