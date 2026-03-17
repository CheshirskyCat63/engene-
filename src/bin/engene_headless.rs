//! ENGENE Headless Binary entrypoint.

fn main() {
    engene::app::headless_runner::run_from_env_args();
}
