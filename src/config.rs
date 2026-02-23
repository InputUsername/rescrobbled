// Copyright (C) 2026 Koen Bolhuis
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

pub mod secrets;

use std::env::{self, VarError};
use std::fs::{self, Permissions};
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::str::FromStr;
use std::time::Duration;

use anyhow::{Context, Result, anyhow, bail};

use regex::RegexSet;

use serde::ser::SerializeSeq;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::config::secrets::{LastFmKey, LastFmSecret, ListenBrainzGlobalToken, ListenBrainzToken};

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

fn deserialize_regex_set<'de, D: Deserializer<'de>>(de: D) -> Result<Option<RegexSet>, D::Error> {
    let set: Vec<String> = Vec::deserialize(de)?;
    RegexSet::new(set)
        .map(Some)
        .map_err(serde::de::Error::custom)
}

fn serialize_regex_set<S: Serializer>(value: &Option<RegexSet>, se: S) -> Result<S::Ok, S::Error> {
    if let Some(s) = value {
        let mut seq = se.serialize_seq(Some(s.len()))?;
        for re in s.patterns() {
            seq.serialize_element(re)?;
        }
        seq.end()
    } else {
        se.serialize_none()
    }
}

#[derive(Deserialize, Serialize, Default, Debug)]
pub struct ListenBrainzConfig {
    pub url: Option<String>,
    #[serde(flatten)]
    pub token: ListenBrainzToken,
}

#[derive(Deserialize, Serialize, Default, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct Config {
    #[serde(flatten)]
    pub lastfm_key: Option<LastFmKey>,
    #[serde(flatten)]
    pub lastfm_secret: Option<LastFmSecret>,
    #[serde(flatten)]
    pub listenbrainz_token: Option<ListenBrainzGlobalToken>,
    #[serde(
        default,
        deserialize_with = "deserialize_duration_seconds",
        serialize_with = "serialize_duration_seconds"
    )]
    pub min_play_time: Option<Duration>,
    #[serde(
        default,
        deserialize_with = "deserialize_regex_set",
        serialize_with = "serialize_regex_set"
    )]
    pub player_whitelist: Option<RegexSet>,
    pub filter_script: Option<PathBuf>,
    pub use_track_start_timestamp: Option<bool>,
    pub listenbrainz: Option<Vec<ListenBrainzConfig>>,
}

impl Config {
    pub fn template() -> String {
        let template = Config {
            lastfm_key: Some(LastFmKey::default()),
            lastfm_secret: Some(LastFmSecret::default()),
            listenbrainz_token: None,
            min_play_time: Some(Duration::from_secs(0)),
            player_whitelist: Some(RegexSet::default()),
            filter_script: Some(PathBuf::new()),
            use_track_start_timestamp: Some(false),
            listenbrainz: Some(vec![ListenBrainzConfig {
                url: Some(String::new()),
                token: ListenBrainzToken::default(),
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
                    token: self.listenbrainz_token.take().unwrap().into(),
                }])
            } else {
                eprintln!(
                    "Warning: both listenbrainz-token and [[listenbrainz]] config options are defined (listenbrainz-token will be ignored)"
                );
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

fn get_envvar<T>(name: &str) -> Result<Option<T>>
where
    T: FromStr,
    <T as FromStr>::Err: std::fmt::Display,
{
    match env::var(name) {
        Ok(value) => value.parse().map(Some).map_err(|err| anyhow!("{err}")),
        Err(VarError::NotPresent) => Ok(None),
        Err(err) => Err(anyhow!("{err}")),
    }
}

fn replace_if_some<T>(option: &mut Option<T>, replacement: Option<T>) {
    if replacement.is_some() {
        *option = replacement;
    }
}

fn override_from_environment(config: &mut Config) -> Result<()> {
    replace_if_some(
        &mut config.lastfm_key,
        get_envvar("LASTFM_KEY")?.map(LastFmKey::Inline),
    );
    replace_if_some(
        &mut config.lastfm_secret,
        get_envvar("LASTFM_SECRET")?.map(LastFmSecret::Inline),
    );
    replace_if_some(
        &mut config.listenbrainz_token,
        get_envvar("LISTENBRAINZ_TOKEN")?.map(ListenBrainzGlobalToken::Inline),
    );
    replace_if_some(
        &mut config.min_play_time,
        get_envvar::<u64>("MIN_PLAY_TIME").map(|t| t.map(Duration::from_secs))?,
    );
    replace_if_some(&mut config.filter_script, get_envvar("FILTER_SCRIPT")?);
    replace_if_some(
        &mut config.use_track_start_timestamp,
        get_envvar("USE_TRACK_START_TIMESTAMP")?,
    );

    Ok(())
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

    override_from_environment(&mut config)?;

    config.normalize();

    Ok(config)
}

#[cfg(test)]
mod tests {
    use std::{borrow::Cow, path::Path, sync::Mutex};

    use crate::config::secrets::Secret;

    use super::*;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

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
        config.listenbrainz_token = Some(ListenBrainzGlobalToken::Inline("TEST TOKEN".to_string()));
        config.normalize();

        assert!(config.listenbrainz_token.is_none());
        assert!(config.listenbrainz.is_some());
        assert!(matches!(
            &config.listenbrainz.unwrap()[..],
            [ListenBrainzConfig { url: None, token }] if token == &ListenBrainzToken::Inline("TEST TOKEN".to_string())
        ));
    }

    #[test]
    fn test_normalize_listenbrainz_double() {
        let mut config = Config::default();
        config.listenbrainz_token = Some(ListenBrainzGlobalToken::Inline("TEST TOKEN".to_string()));
        config.listenbrainz = Some(vec![ListenBrainzConfig {
            url: None,
            token: ListenBrainzToken::Inline("SECOND TEST TOKEN".to_string()),
        }]);
        config.normalize();

        assert!(config.listenbrainz_token.is_none());
        assert!(config.listenbrainz.is_some());
    }

    #[test]
    fn test_override_from_environment() {
        let mut config = Config::default();

        let _guard = ENV_LOCK.lock().unwrap();

        // Safety: a mutex is used to ensure this test runs single-threaded.
        // No other test uses environment variables.
        unsafe {
            std::env::set_var("LASTFM_KEY", "lastfm_key_123");
            std::env::set_var("LASTFM_SECRET", "lastfm_secret_456");
            std::env::set_var("LISTENBRAINZ_TOKEN", "listenbrainz_token_xyz");
            std::env::set_var("MIN_PLAY_TIME", "30");
            std::env::set_var("FILTER_SCRIPT", "/tmp/filter.sh");
            std::env::set_var("USE_TRACK_START_TIMESTAMP", "true");
        }

        override_from_environment(&mut config).unwrap();

        assert_eq!(
            config.lastfm_key,
            Some(LastFmKey::Inline("lastfm_key_123".to_string()))
        );
        assert_eq!(
            config.lastfm_secret,
            Some(LastFmSecret::Inline("lastfm_secret_456".to_string()))
        );
        assert_eq!(
            config.listenbrainz_token,
            Some(ListenBrainzGlobalToken::Inline(
                "listenbrainz_token_xyz".to_string()
            ))
        );
        assert_eq!(config.min_play_time, Some(Duration::from_secs(30)));
        assert_eq!(
            config.filter_script.as_deref(),
            Some(Path::new("/tmp/filter.sh"))
        );
        assert_eq!(config.use_track_start_timestamp, Some(true));
    }

    #[test]
    fn test_secrets_from_file() {
        assert_eq!(
            LastFmKey::File("tests/secret".to_string()).get().unwrap(),
            Cow::<str>::Owned("something secret".to_string())
        )
    }
}
