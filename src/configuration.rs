use config::{Config, ConfigError, Environment, File};
use std::path::PathBuf;
use serde::Deserialize;

#[derive(Deserialize, Clone)]
pub struct Configuration {
    #[serde(default = "default_environment")]
    pub environment: String,
    #[serde(default = "default_server_port")]
    pub server_port: u16,
    #[serde(default = "default_log_level")]
    pub log_level: String,
    #[serde(default)]
    pub app_domain: String, // used to set CORS
    pub database_connection_string: String,
    #[serde(default)]
    pub run_database_migrations: bool,
    #[serde(default)]
    pub secrets: Secrets,
    #[serde(default)]
    pub jobs: Jobs,
}

#[derive(Deserialize, Clone, Default)]
pub struct Secrets {
    #[serde(default)]
    pub coingecko_api_key: String,
}

#[derive(Deserialize, Clone)]
pub struct Jobs {
    #[serde(default = "default_update_exchange_rate_cron")]
    pub update_exchange_rate_cron: String,
}

impl Default for Jobs {
    fn default() -> Self {
        Self {
            update_exchange_rate_cron: default_update_exchange_rate_cron(),
        }
    }
}

fn default_environment() -> String {
    "Dev".to_string()
}

fn default_server_port() -> u16 {
    3000
}

fn default_log_level() -> String {
    "info".to_string()
}

fn default_update_exchange_rate_cron() -> String {
    "0 */10 * * * *".to_string()
}

impl Configuration {
    /// Layered precedence: env vars (prefix `APP`, nested keys separated by `__`) > JSON file > built-in defaults.
    /// The JSON file path comes from the optional `CONFIGURATION_FILE` variable; when unset or missing only env + defaults apply.
    pub fn load() -> Result<Self, ConfigError> {
        let mut builder = Config::builder();

        if let Ok(file) = std::env::var("CONFIGURATION_FILE") {
            builder = builder.add_source(File::from(PathBuf::from(file)).required(false));
        }

        // prefix_separator must be explicit: otherwise it falls back to the key separator
        // (`__`) and only `APP__*` variables would match.
        builder = builder.add_source(
            Environment::with_prefix("APP")
                .prefix_separator("_")
                .separator("__"),
        );

        builder.build()?.try_deserialize::<Self>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn temp_config_file(json: &str) -> String {
        let mut path = std::env::temp_dir();
        path.push(format!("cfg_test_{}.json", std::process::id()));
        let mut file = std::fs::File::create(&path).expect("failed to create temp config");
        file.write_all(json.as_bytes()).expect("failed to write temp config");
        path.to_string_lossy().to_string()
    }

    // Single test (not two): both scenarios mutate the global CONFIGURATION_FILE variable.
    #[test]
    fn layered_precedence_env_beats_file_beats_defaults() {
        let file = temp_config_file(
            r#"{"database_connection_string":"postgres://x","server_port":4000}"#,
        );
        std::env::set_var("CONFIGURATION_FILE", &file);
        std::env::set_var("APP_SERVER_PORT", "5000");

        let c = Configuration::load().expect("configuration should load");

        assert_eq!(c.server_port, 5000, "env must win over file");
        assert_eq!(c.environment, "Dev", "missing field falls back to default");
        assert_eq!(c.log_level, "info");
        assert!(!c.run_database_migrations);
        assert_eq!(c.jobs.update_exchange_rate_cron, "0 */10 * * * *");

        std::env::remove_var("CONFIGURATION_FILE");
        std::env::remove_var("APP_SERVER_PORT");
        let _ = std::fs::remove_file(&file);

        let file = temp_config_file(
            r#"{"database_connection_string":"postgres://x","server_port":4000,"environment":"Prod"}"#,
        );
        std::env::set_var("CONFIGURATION_FILE", &file);

        let c = Configuration::load().expect("configuration should load");

        assert_eq!(c.server_port, 4000);
        assert_eq!(c.environment, "Prod");

        std::env::remove_var("CONFIGURATION_FILE");
        let _ = std::fs::remove_file(&file);
    }
}
