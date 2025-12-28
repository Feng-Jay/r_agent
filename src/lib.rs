pub mod model;
pub mod config;
pub fn test_logging() {
    tracing::info!("This is a test log from the library.");
}