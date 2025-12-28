use std::{collections::HashMap, env};
use anyhow::Context;
use std::path::{Path, PathBuf};
use serde::Deserialize;

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
    pub debug: bool,
    pub models: HashMap<String, ModelConfig>,
    pub summary_model: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ModelConfig {
    pub api_key: String,
    pub base_url: Option<String>,
    pub cost: Option<Cost>,
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
    let cfg = serde_yml::from_str(&config_content)
                                .with_context(|| format!("Failed to parse config file: {}", config_path.display()))
                                .unwrap();
    cfg
}

mod tests {
    use super::*;
    #[test]
    fn test_load_config() {
        let config = load_config(None);
        println!("{:?}", config);
        assert!(config.models.len() > 0);
    }
}