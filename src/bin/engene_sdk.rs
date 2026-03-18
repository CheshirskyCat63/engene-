//! ENGENE SDK binary bootstrap entrypoint.

fn main() {
    engene::app::sdk_runner::run_from_env_args();
}
