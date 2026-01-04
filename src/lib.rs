pub mod model;
pub mod config;
pub mod agent;
pub mod memory;
pub mod prompt;
pub mod tool;

pub fn test_logging() {
    tracing::info!("This is a test log from the library.");
}