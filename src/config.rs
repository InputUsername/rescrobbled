use std::fmt;
use std::fs;
use std::io;
use std::time::Duration;

use serde::Deserialize;

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Config {
    pub api_key: String,
    pub api_secret: String,

    pub min_play_time: Option<Duration>,
}

pub enum ConfigError {
    Io(io::Error),
    Format(String),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ConfigError::Io(err) => write!(f, "{}", err),
            ConfigError::Format(msg) => write!(f, "{}", msg),
        }
    }
}

pub fn load_config() -> Result<Config, ConfigError> {
    let buffer = fs::read_to_string("config.toml")
        .map_err(|err| ConfigError::Io(err))?;

    toml::from_str(&buffer)
        .map_err(|err| ConfigError::Format(format!("Could not parse config: {}", err)))
}