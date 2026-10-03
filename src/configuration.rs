use serde::Deserialize;
use std::fs;

#[derive(Deserialize, Clone)]
pub struct Configuration {
    pub environment: String,
    pub server_port: u16,
    pub log_level: String,
    pub app_domain: String, // used to set CORS
    pub database_connection_string: String,
    pub database_pool: DatabasePool,
    pub run_database_migrations: bool,
    pub secrets: Secrets,
    pub jobs: Jobs,
}

#[derive(Deserialize, Clone)]
pub struct DatabasePool {
    pub max_connections: u32,
    pub min_connections: u32,
    pub acquire_timeout_secs: u64,
}

#[derive(Deserialize, Clone)]
pub struct Secrets {
    pub coingecko_api_key: String,
}

#[derive(Deserialize, Clone)]
pub struct Jobs {
    pub update_exchange_rate_cron: String,
}

impl Configuration {
    pub fn load_from_json_file(file: &str) -> Result<Configuration, String> {
        let content = fs::read_to_string(file)
            .map_err(|e| format!("Failed to read configuration file '{}': {}", file, e))?;

        let config: Configuration = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to deserialize configuration file '{}': {}", file, e))?;

        Ok(config)
    }
}
