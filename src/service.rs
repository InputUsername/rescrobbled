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

use anyhow::{Context, Result, anyhow};

use rustfm_scrobble::Scrobbler;

mod lastfm;

use crate::config::Config;

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
}
