use std::{collections::HashMap, env};
use anyhow::Context;
use std::path::PathBuf;
use serde::Deserialize;
use tracing_subscriber::{fmt, EnvFilter};
use tracing_appender::{rolling, non_blocking::WorkerGuard};
use once_cell::sync::OnceCell;
static LOG_GUARD: OnceCell<WorkerGuard> = OnceCell::new();

pub fn get_config_file_path() -> PathBuf {
    env::var("R_AGENT_CONFIG_FILE")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            let mut default_path = env::current_dir().expect("Failed to get current directory");
            default_path.push("config.yaml");
            default_path
        })
}

#[derive(Deserialize, Debug)]
pub struct Config {
    pub log_level: String,
    pub log_dir: String,
    pub log_file: String,
    pub models: HashMap<String, ModelConfig>,
    pub summary_model: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ModelConfig {
    pub api_key: String,
    pub base_url: Option<String>,
    pub cost: Option<Cost>,
    pub temperature: Option<f32>,
    pub top_p: Option<f32>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Cost{
    pub input_cost_per_token: f64,
    pub output_cost_per_token: f64,
    pub max_tokens: usize,
    pub max_input_tokens: usize,
    pub max_output_tokens: usize,
}

pub fn load_config(file: Option<&str>) -> Config {
    let config_path = match file {
        Some(path) => PathBuf::from(path),
        None => get_config_file_path(),
    };

    let config_content = std::fs::read_to_string(&config_path)
                                .with_context(|| format!("Failed to read config file: {}", config_path.display()))
                                .unwrap();
    let cfg: Config = serde_yml::from_str(&config_content)
                                .with_context(|| format!("Failed to parse config file: {}", config_path.display()))
                                .unwrap();
    // init the logging
    LOG_GUARD.get_or_init(|| 
        {
            let appender = rolling::never(cfg.log_dir.clone(), cfg.log_file.clone());
            let (writer, guard) = tracing_appender::non_blocking(appender);
            let filter = EnvFilter::new(cfg.log_level.clone());
            let _ = fmt()
                        .with_env_filter(filter)
                        .with_writer(writer)
                        .with_ansi(false)
                        .init();
            guard
        });
    // return the config
    cfg
}

mod tests {
    #[test]
    fn test_load_config() {
        let config = super::load_config(None);
        println!("{:?}", config);
        tracing::debug!("Config loaded successfully: {:?}", config);
        assert!(config.models.len() > 0);
    }
}