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
use std::fs::{self, Permissions};
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::time::Duration;

use anyhow::{anyhow, bail, Context, Result};

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
pub struct ListenBrainzConfig {
    pub url: Option<String>,
    pub token: String,
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

    pub listenbrainz: Option<Vec<ListenBrainzConfig>>,
}

impl Config {
    pub fn template() -> String {
        let template = Config {
            lastfm_key: Some(String::new()),
            lastfm_secret: Some(String::new()),
            listenbrainz_token: None,
            enable_notifications: Some(false),
            min_play_time: Some(Duration::from_secs(0)),
            player_whitelist: Some(HashSet::new()),
            filter_script: Some(PathBuf::new()),
            listenbrainz: Some(vec![ListenBrainzConfig {
                url: Some(String::new()),
                token: String::new(),
            }]),
        };
        toml::to_string(&template)
            .unwrap()
            .lines()
            .map(|l| format!("# {}\n", l))
            .collect()
    }

    fn normalize(&mut self) {
        // Turn `listenbrainz-token` into a `[[listenbrainz]]` definition
        if self.listenbrainz_token.is_some() {
            if self.listenbrainz.is_none() {
                self.listenbrainz = Some(vec![ListenBrainzConfig {
                    url: None,
                    token: self.listenbrainz_token.take().unwrap(),
                }])
            } else {
                eprintln!("Warning: both listenbrainz-token and [[listenbrainz]] config options are defined (listenbrainz-token will be ignored)");
            }

            self.listenbrainz_token.take();
        }
    }
}

pub fn config_dir() -> Result<PathBuf> {
    let mut path =
        dirs::config_dir().ok_or_else(|| anyhow!("User config directory does not exist"))?;

    path.push(CONFIG_DIR);

    if !path.exists() {
        fs::create_dir_all(&path).context("Failed to create config directory")?;
    }

    Ok(path)
}

pub fn load_config() -> Result<Config> {
    let mut path = config_dir()?;

    path.push(CONFIG_FILE);

    if !path.exists() {
        fs::write(&path, Config::template()).context("Failed to create config template")?;
        fs::set_permissions(&path, Permissions::from_mode(0o600))
            .context("Failed to set permissions for config file")?;

        bail!(
            "Config file did not exist, created it at {}",
            path.display()
        );
    }

    let buffer = fs::read_to_string(&path).context("Failed to open config file")?;

    let mut config: Config = toml::from_str(&buffer).context("Failed to parse config file")?;

    config.normalize();

    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_empty_config() {
        let mut config = Config::default();
        config.normalize();

        assert!(config.listenbrainz_token.is_none());
        assert!(config.listenbrainz.is_none());
    }

    #[test]
    fn test_normalize_listenbrainz_token() {
        let mut config = Config::default();
        config.listenbrainz_token = Some("TEST TOKEN".to_string());
        config.normalize();

        assert!(config.listenbrainz_token.is_none());
        assert!(config.listenbrainz.is_some());
        assert!(
            matches!(
                &config.listenbrainz.unwrap()[..],
                [ListenBrainzConfig { url: None, token }] if token == "TEST TOKEN"
            )
        );
    }

    #[test]
    fn test_normalize_listenbrainz_double() {
        let mut config = Config::default();
        config.listenbrainz_token = Some("TEST TOKEN".to_string());
        config.listenbrainz = Some(vec![ListenBrainzConfig {
            url: None,
            token: "SECOND TEST TOKEN".to_string(),
        }]);
        config.normalize();

        assert!(config.listenbrainz_token.is_none());
        assert!(config.listenbrainz.is_some());
    }
}
