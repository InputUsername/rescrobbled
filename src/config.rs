// Copyright (C) 2021 Koen Bolhuis
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

use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;
use std::time::Duration;

use anyhow::{Context, Result, anyhow};

use serde::{Deserialize, Deserializer, Serialize, Serializer};

const CONFIG_DIR: &str = "rescrobbled";
const CONFIG_FILE: &str = "config.toml";

fn deserialize_duration_seconds<'de, D: Deserializer<'de>>(
    de: D,
) -> Result<Option<Duration>, D::Error> {
    Ok(Some(Duration::from_secs(u64::deserialize(de)?)))
}

fn serialize_duration_seconds<S: Serializer>(
    value: &Option<Duration>,
    se: S,
) -> Result<S::Ok, S::Error> {
    if let Some(d) = value {
        se.serialize_some(&d.as_secs())
    } else {
        se.serialize_none()
    }
}

#[derive(Deserialize, Serialize, Default, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct Config {
    #[serde(alias = "api-key")]
    pub lastfm_key: Option<String>,

    #[serde(alias = "api-secret")]
    pub lastfm_secret: Option<String>,

    #[serde(alias = "lb-token")]
    pub listenbrainz_token: Option<String>,

    pub enable_notifications: Option<bool>,

    #[serde(
        default,
        deserialize_with = "deserialize_duration_seconds",
        serialize_with = "serialize_duration_seconds"
    )]
    pub min_play_time: Option<Duration>,

    pub player_whitelist: Option<HashSet<String>>,

    pub filter_script: Option<PathBuf>,
}

impl Config {
    pub fn template() -> String {
        let template = Config {
            lastfm_key: Some(String::new()),
            lastfm_secret: Some(String::new()),
            listenbrainz_token: Some(String::new()),
            enable_notifications: Some(false),
            min_play_time: Some(Duration::from_secs(0)),
            player_whitelist: Some(HashSet::new()),
            filter_script: Some(PathBuf::new()),
        };
        toml::to_string(&template)
            .unwrap()
            .lines()
            .map(|l| format!("# {}\n", l))
            .collect()
    }
}

pub fn config_dir() -> Result<PathBuf> {
    let mut path = dirs::config_dir()
        .ok_or_else(|| anyhow!("User config directory does not exist"))?;

    path.push(CONFIG_DIR);

    if !path.exists() {
        fs::create_dir_all(&path)
            .context("Failed to create config directory")?;
    }

    Ok(path)
}

pub fn load_config() -> Result<Config> {
    let mut path = config_dir()?;

    path.push(CONFIG_FILE);

    if !path.exists() {
        fs::write(&path, Config::template())
            .context("Failed to create config template")?;

        return Err(anyhow!("Config file did not exist, created it at {}", path.display()));
    }

    let buffer = fs::read_to_string(&path)
        .context("Failed to open config file")?;

    toml::from_str(&buffer)
        .context("Failed to parse config file")
}
