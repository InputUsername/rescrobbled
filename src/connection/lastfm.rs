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
use std::fmt::Display;
use std::fs::{self, Permissions};
use std::io::{self, Write};
use std::os::unix::fs::PermissionsExt;
use std::time::UNIX_EPOCH;

use anyhow::{anyhow, Context, Result};

use rustfm_scrobble_proxy::{Scrobble, Scrobbler};

use rpassword::read_password;

use crate::config::{config_dir, Config};
use crate::connection::{lastfm, ServiceConnection};

const SESSION_FILE: &str = "session";

/// Authenticate with Last.fm either using an existing
/// session file or by logging in.
pub fn authenticate(scrobbler: &mut Scrobbler) -> Result<()> {
    let mut path = config_dir()?;
    path.push(SESSION_FILE);

    if let Ok(session_key) = fs::read_to_string(&path) {
        // TODO: validate session
        scrobbler.authenticate_with_session_key(&session_key);
    } else {
        let mut input = String::new();

        print!(
            "Log in to Last.fm\n\
            Username: "
        );
        io::stdout().flush()?;

        io::stdin().read_line(&mut input)?;
        input.pop();
        let username = input.clone();

        input.clear();

        print!("Password: ");
        io::stdout().flush()?;

        let password = read_password().context("Failed to read password")?;

        let session_response = scrobbler.authenticate_with_password(&username, &password)?;

        let _ = fs::write(&path, session_response.key);
        let _ = fs::set_permissions(&path, Permissions::from_mode(0o600));
    }

    Ok(())
}

pub struct LastFMConnection {
    scrobbler: Scrobbler,
}

impl LastFMConnection {
    /// Try to connect to Last.fm.
    pub fn new(config: &Config) -> Result<Self> {
        match (&config.lastfm_key, &config.lastfm_secret) {
            (Some(key), Some(secret)) => {
                let mut scrobbler = Scrobbler::new(key, secret);

                lastfm::authenticate(&mut scrobbler)
                    .context("Failed to authenticate with Last.fm")?;

                Ok(Self { scrobbler })
            }
            _ => Err(anyhow!("Last.fm API key or API secret are missing")),
        }
    }

    /// Return true if connecting to Last.fm should be attempted
    pub fn is_configured(config: &Config) -> bool {
        config.lastfm_key.is_some() || config.lastfm_secret.is_some()
    }
}

impl ServiceConnection for LastFMConnection {
    fn now_playing(&self, track: &crate::track::Track) -> Result<()> {
        let scrobble = Scrobble::new(track.artist(), track.title(), track.album());

        self.scrobbler
            .now_playing(&scrobble)
            .with_context(|| format!("Failed to update status on {}", self))?;

        Ok(())
    }

    fn submit(
        &self,
        track: &crate::track::Track,
        track_start: Option<&std::time::SystemTime>,
    ) -> Result<()> {
        let mut scrobble = Scrobble::new(track.artist(), track.title(), track.album());

        if let Some(track_start) = track_start {
            let timestamp = track_start
                .duration_since(UNIX_EPOCH)
                .context("Track started before UNIX epoch")?;

            scrobble.with_timestamp(timestamp.as_secs());
        }

        self.scrobbler
            .scrobble(&scrobble)
            .with_context(|| format!("Failed to submit track to {}", self))?;

        Ok(())
    }
}

impl Display for LastFMConnection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "last.fm")
    }
}
