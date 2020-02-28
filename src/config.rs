// Copyright (C) 2019 Koen Bolhuis
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use std::fmt;
use std::fs;
use std::io;
use std::time::Duration;
use std::collections::HashSet;

use dirs;

use serde::{Deserialize, Deserializer};

fn deserialize_duration_seconds<'de, D: Deserializer<'de>>(de: D) -> Result<Option<Duration>, D::Error> {
    Ok(Some(Duration::from_secs(u64::deserialize(de)?)))
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Config {
    #[serde(alias = "api-key")]
    pub lastfm_key: String,

    #[serde(alias = "api-secret")]
    pub lastfm_secret: String,

    #[serde(alias = "lb-token")]
    pub listenbrainz_token: Option<String>,

    pub enable_notifications: Option<bool>,

    #[serde(default, deserialize_with = "deserialize_duration_seconds")]
    pub min_play_time: Option<Duration>,

    pub player_whitelist: Option<HashSet<String>>,
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
    let mut path = dirs::config_dir()
        .ok_or(ConfigError::Io(io::Error::new(io::ErrorKind::NotFound, "User config directory not found")))?;

    path.push("rescrobbled");

    fs::create_dir_all(&path)
        .map_err(|err| ConfigError::Io(err))?;

    path.push("config.toml");

    let buffer = fs::read_to_string(&path)
        .map_err(|err| ConfigError::Io(err))?;

    toml::from_str(&buffer)
        .map_err(|err| ConfigError::Format(format!("Could not parse config: {}", err)))
}
