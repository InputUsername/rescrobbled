use std::{borrow::Cow, fs, path::PathBuf};

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub enum ListenBrainzToken {
    #[serde(rename = "token")]
    Inline(String),
    #[serde(rename = "token-file")]
    File(String),
}

impl Default for ListenBrainzToken {
    fn default() -> Self {
        Self::Inline(String::default())
    }
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub enum ListenBrainzGlobalToken {
    #[serde(rename = "listenbrainz-token")]
    #[serde(alias = "lb-token")]
    Inline(String),
    #[serde(rename = "listenbrainz-token-file")]
    File(String),
}

impl Default for ListenBrainzGlobalToken {
    fn default() -> Self {
        Self::Inline(String::default())
    }
}

impl From<ListenBrainzGlobalToken> for ListenBrainzToken {
    fn from(value: ListenBrainzGlobalToken) -> Self {
        match value {
            ListenBrainzGlobalToken::Inline(v) => ListenBrainzToken::Inline(v),
            ListenBrainzGlobalToken::File(v) => ListenBrainzToken::File(v),
        }
    }
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub enum LastFmKey {
    #[serde(rename = "lastfm-key")]
    #[serde(alias = "api-key")]
    Inline(String),
    #[serde(rename = "lastfm-key-file")]
    File(String),
}

impl Default for LastFmKey {
    fn default() -> Self {
        Self::Inline(String::default())
    }
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub enum LastFmSecret {
    #[serde(rename = "lastfm-secret")]
    #[serde(alias = "api-secret")]
    Inline(String),
    #[serde(rename = "lastfm-secret-file")]
    File(String),
}

impl Default for LastFmSecret {
    fn default() -> Self {
        Self::Inline(String::default())
    }
}

pub trait Secret {
    /// Returns the token if it's `Inline`, or fetches it from the specified `File`.
    fn get(&'_ self) -> Result<Cow<'_, str>>;
}

// Because using derive macros is too complicated, and converting them all to a `Secret` enum is
// difficult:
// - At deserialization, as some tokens are optional, hence is blocked by
//   https://github.com/serde-rs/serde/issues/723
// - When creating request clients, as there are lifetime issues
macro_rules! impl_secret {
    ($name:ident) => {
        impl Secret for $name {
            fn get(&'_ self) -> Result<Cow<'_, str>> {
                match self {
                    $name::Inline(secret) => Ok(Cow::Borrowed(secret)),
                    $name::File(path) => {
                        let path = if let Some(stripped) = path.strip_prefix("~/") {
                            dirs::home_dir()
                                .ok_or_else(|| anyhow!("User home directory does not exist"))?
                                .join(stripped)
                        } else {
                            PathBuf::from(path)
                        };
                        let secret = fs::read_to_string(path)?;

                        Ok(Cow::Owned(secret.trim().to_owned()))
                    }
                }
            }
        }
    };
}

impl_secret!(ListenBrainzToken);
impl_secret!(ListenBrainzGlobalToken);
impl_secret!(LastFmKey);
impl_secret!(LastFmSecret);
