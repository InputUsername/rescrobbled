// Copyright (C) 2025 Koen Bolhuis
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
use anyhow::{Context, Result};
use listenbrainz::ListenBrainz;
use std::fmt::{self, Display, Formatter, Write};
use std::time::SystemTime;

use crate::track::Track;
use crate::{config::ListenBrainzConfig, service::Service};

pub struct ListenBrainzService {
    client: ListenBrainz,
    is_default: bool,
}

impl ListenBrainzService {
    /// Try to connect to a ListenBrainz instance.
    pub fn new(lb: &ListenBrainzConfig) -> Result<Self> {
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

        Ok(Self {
            is_default: lb.url.is_none(),
            client,
        })
    }
}
impl Service for ListenBrainzService {
    fn now_playing(&self, track: &Track) -> Result<()> {
        self.client
            .playing_now(track.artist(), track.title(), track.album())
            .with_context(|| format!("Failed to update status on {}", self))?;
        Ok(())
    }

    fn submit(&self, track: &Track, _track_start: Option<&SystemTime>) -> Result<()> {
        self.client
            .listen(track.artist(), track.title(), track.album())
            .with_context(|| format!("Failed to submit track to {}", self))?;
        Ok(())
    }
}

impl Display for ListenBrainzService {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "ListenBrainz")?;
        if !self.is_default {
            write!(f, " ({})", self.client.api_url())?;
        }
        Ok(())
    }
}
