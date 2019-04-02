use std::fmt;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use toml::Value;

pub struct Config {
    pub api_key: String,
    pub api_secret: String,
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
    let mut file = match File::open("config.toml") {
        Ok(file) => file,
        Err(err) => return Err(ConfigError::Io(err)),
    };

    let mut buffer = String::new();

    if let Err(err) = file.read_to_string(&mut buffer) {
        return Err(ConfigError::Io(err));
    }

    let value = match buffer.parse::<Value>() {
        Ok(value) => value,
        Err(_) => return Err(ConfigError::Format("Could not parse config as TOML".to_string())),
    };

    if !value["api-key"].is_str() {
        return Err(ConfigError::Format("API key is not a string".to_string()));
    }
    if !value["api-secret"].is_str() {
        return Err(ConfigError::Format("API secret is not a string".to_string()));
    }

    let key = value["api-key"].as_str().unwrap().to_string();
    let secret = value["api-secret"].as_str().unwrap().to_string();

    Ok(Config {
        api_key: key,
        api_secret: secret,
    })
}