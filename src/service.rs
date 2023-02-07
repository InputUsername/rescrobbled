// Copyright (C) 2023 Koen Bolhuis
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

use std::fmt::{self, Write};

use anyhow::{anyhow, Context, Result};

use listenbrainz::ListenBrainz;

use rustfm_scrobble_proxy::{Scrobble, Scrobbler};

use tracing::{error, info, warn};

mod lastfm;

use crate::config::{Config, ListenBrainzConfig};
use crate::track::Track;

/// Represents a music scrobbling service.
pub enum Service {
    LastFM(Scrobbler),
    ListenBrainz {
        client: ListenBrainz,
        is_default: bool,
    },
}

impl Service {
    /// Try to connect to Last.fm.
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

    /// Try to connect to a ListenBrainz instance.
    fn listenbrainz(lb: &ListenBrainzConfig) -> Result<Self> {
        let mut client = match lb.url {
            Some(ref url) => ListenBrainz::new_with_url(url),
            None => ListenBrainz::new(),
        };

        client.authenticate(&lb.token).with_context(|| {
            let mut err = "Failed to authenticate with ListenBrainz".to_owned();
            if let Some(ref url) = lb.url {
                write!(err, " ({url})").unwrap();
            }
            err
        })?;

        Ok(Self::ListenBrainz {
            is_default: lb.url.is_none(),
            client,
        })
    }

    /// Initialize all services specified in the config.
    pub fn initialize_all(config: &Config) -> Vec<Self> {
        let mut services = Vec::new();

        match Self::lastfm(config) {
            Ok(Some(lastfm)) => {
                info!("Authenticated with {} successfully!", lastfm);
                services.push(lastfm);
            }
            Err(err) => error!("{:?}", err),
            _ => {}
        }

        for lb in config.listenbrainz.iter().flatten() {
            match Self::listenbrainz(lb) {
                Ok(service) => {
                    info!("Authenticated with {} successfully!", service);
                    services.push(service);
                }
                Err(err) => error!("{:?}", err),
            }
        }

        if services.is_empty() {
            warn!("No scrobbling services defined");
        }

        services
    }

    /// Submit a "now playing" request.
    pub fn now_playing(&self, track: &Track) -> Result<()> {
        match self {
            Self::LastFM(scrobbler) => {
                let album = track
                    .album()
                    .ok_or_else(|| anyhow!("Cannot submit track without album to {self}"))?;

                let scrobble = Scrobble::new(track.artist(), track.title(), album);

                scrobbler
                    .now_playing(&scrobble)
                    .with_context(|| format!("Failed to update status on {}", self))?;
            }
            Self::ListenBrainz { client, .. } => {
                client
                    .playing_now(track.artist(), track.title(), track.album())
                    .with_context(|| format!("Failed to update status on {}", self))?;
            }
        }
        Ok(())
    }

    /// Scrobble a track.
    pub fn submit(&self, track: &Track) -> Result<()> {
        match self {
            Self::LastFM(scrobbler) => {
                let album = track
                    .album()
                    .ok_or_else(|| anyhow!("Cannot submit track without album to {self}"))?;

                let scrobble = Scrobble::new(track.artist(), track.title(), album);

                scrobbler
                    .scrobble(&scrobble)
                    .with_context(|| format!("Failed to submit track to {}", self))?;
            }
            Self::ListenBrainz { client, .. } => {
                client
                    .listen(track.artist(), track.title(), track.album())
                    .with_context(|| format!("Failed to submit track to {}", self))?;
            }
        }
        Ok(())
    }
}

impl fmt::Display for Service {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LastFM(_) => write!(f, "Last.fm"),
            Self::ListenBrainz { client, is_default } => {
                write!(f, "ListenBrainz")?;
                if !is_default {
                    write!(f, " ({})", client.api_url())?;
                }
                Ok(())
            }
        }
    }
}
