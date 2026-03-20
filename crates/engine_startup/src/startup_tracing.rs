use std::sync::Once;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use tracing_subscriber::fmt::Layer;
use tracing_subscriber::EnvFilter;

static INIT_TRACING: Once = Once::new();

pub fn startup_tracing() {
    INIT_TRACING.call_once(|| {
        let fmt_layer = Layer::new();
        let filter_layer = EnvFilter::try_from_default_env()
            .or_else(|_| EnvFilter::try_new("info"))
            .unwrap();
        
        tracing_subscriber::registry()
            .with(filter_layer)
            .with(fmt_layer)
            .init();
            
        tracing::info!("Startup tracing initialized");
    });
}