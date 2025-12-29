pub mod model;
pub mod config;
pub mod agent;
pub mod memory;

use tracing_subscriber::{fmt, EnvFilter};
use tracing::{debug, info};
pub fn test_logging() {
    tracing::info!("This is a test log from the library.");
}