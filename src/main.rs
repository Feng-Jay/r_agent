use tracing_subscriber::{fmt, EnvFilter};
use r_agent::model::base::BaseModel;
use r_agent::model::litellm_model::Litellm_Model;
use r_agent::test_logging;
use r_agent::config::*;

fn init_logging() {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));
    fmt().with_env_filter(filter).init();
}

fn main() {
    println!("Hello, world!");
    init_logging();
    tracing::info!("Logging is initialized.");
    test_logging();
}
