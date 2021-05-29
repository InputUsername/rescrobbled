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

use std::fmt;

use anyhow::{Context, Result, anyhow, bail};

use listenbrainz_rust::Listen;

use rustfm_scrobble::Scrobbler;

mod lastfm;

use crate::config::Config;
use crate::track::Track;

/// Represents a music scrobbling service.
pub enum Service {
    LastFM(Scrobbler),
    ListenBrainz(String)
}

impl Service {
    /// Initialize Last.fm.
    fn lastfm(config: &Config) -> Result<Option<Self>> {
        match (&config.lastfm_key, &config.lastfm_secret) {
            (Some(key), Some(secret)) => {
                let mut scrobbler = Scrobbler::new(key, secret);

                lastfm::authenticate(&mut scrobbler)
                    .context("Failed to authenticate with Last.fm")?;

                Ok(Some(Self::LastFM(scrobbler)))
            }
            (None, None) => Ok(None),
            _ => Err(anyhow!("Last.fm API key or API secret are missing")),
        }
    }

    /// Initialize ListenBrainz.
    fn listenbrainz(config: &Config) -> Option<Self> {
        config.listenbrainz_token.clone().map(Self::ListenBrainz)
    }

    /// Initialize all services specified in the config.
    pub fn initialize_all(config: &Config) -> Result<Vec<Self>> {
        let mut services = Vec::new();

        if let Some(lastfm) = Self::lastfm(config)? {
            services.push(lastfm);
            println!("Authenticated with Last.fm successfully!");
        }
        if let Some(listenbrainz) = Self::listenbrainz(config) {
            services.push(listenbrainz);
        }

        Ok(services)
    }

    /// Submit a "now playing" request.
    pub fn now_playing(&self, track: &Track) -> Result<()> {
        match self {
            Self::LastFM(scrobbler) => {
                scrobbler.now_playing(&track.into())
                    .context("Failed to update status on Last.fm")?;
            }
            Self::ListenBrainz(token) => {
                let status = Listen::from(track).playing_now(token)
                    .map_err(|err| anyhow!("{}", err))
                    .context("Failed to update status on ListenBrainz")?;

                if !status.is_success() {
                    // TODO: remove this check when a new ListenBrainz library is
                    // used - error responses should not require checking the HTTP status
                    bail!("Failed to update status on ListenBrainz");
                }
            }
        }
        Ok(())
    }

    /// Scrobble a track.
    pub fn submit(&self, track: &Track) -> Result<()> {
        match self {
            Self::LastFM(scrobbler) => {
                scrobbler.scrobble(&track.into())
                    .context("Failed to submit track to Last.fm")?;
            }
            Self::ListenBrainz(token) => {
                let status = Listen::from(track).single(token)
                    .map_err(|err| anyhow!("{}", err))
                    .context("Failed to submit track to ListenBrainz")?;

                if !status.is_success() {
                    // TODO: remove this check when a new ListenBrainz library is
                    // used - error responses should not require checking the HTTP status
                    bail!("Failed to submit track to ListenBrainz");
                }
            }
        }
        Ok(())
    }
}

impl fmt::Display for Service {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::LastFM(_) => write!(f, "Last.fm"),
            Self::ListenBrainz(_) => write!(f, "ListenBrainz"),
        }
    }
}
