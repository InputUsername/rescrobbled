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

use std::fmt::Display;
use std::time::SystemTime;

use anyhow::Result;

use crate::service::{LastFMService, ListenBrainzService};

use crate::config::Config;
use crate::track::Track;

pub trait Service: Display {
    /// Submit a "now playing" request.
    fn now_playing(&self, track: &Track) -> Result<()>;
    /// Scrobble a track.
    fn submit(&self, track: &Track, track_start: Option<&SystemTime>) -> Result<()>;
}

/// Initialize all services specified in the config.
pub fn initialize_all(config: &Config) -> Vec<Box<dyn Service>> {
    let mut services: Vec<Box<dyn Service>> = Vec::new();

    match LastFMService::new(config) {
        Ok(Some(lastfm)) => {
            println!("Authenticated with {} successfully!", lastfm);
            services.push(Box::new(lastfm));
        }
        Err(err) => eprintln!("{:?}", err),
        _ => {}
    }

    for lb in config.listenbrainz.iter().flatten() {
        match ListenBrainzService::new(lb) {
            Ok(service) => {
                println!("Authenticated with {} successfully!", service);
                services.push(Box::new(service));
            }
            Err(err) => eprintln!("{:?}", err),
        }
    }

    if services.is_empty() {
        eprintln!("Warning: no scrobbling services defined");
    }

    services
}
