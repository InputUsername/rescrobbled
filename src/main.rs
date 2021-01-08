// Rescrobbled is a simple music scrobbler daemon.
//
// Copyright (C) 2019 Koen Bolhuis
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

use std::process;

use rustfm_scrobble::Scrobbler;

mod auth;
mod config;
mod mainloop;

use config::ConfigError;

fn main() {
    let config = match config::load_config() {
        Ok(config) => config,
        Err(ConfigError::Created(path)) => {
            println!(
                "Config file did not exist; created it at {}\n\
                Please update it with your Last.fm/ListenBrainz API information.",
                path.to_string_lossy()
            );
            return;
        }
        Err(err) => {
            eprintln!("Error while loading config: {}", err);
            process::exit(1);
        }
    };

    let scrobbler = match (&config.lastfm_key, &config.lastfm_secret) {
        (Some(key), Some(secret)) => {
            let mut scrobbler = Scrobbler::new(key, secret);
            match auth::authenticate(&mut scrobbler) {
                Ok(_) => println!("Authenticated with Last.fm successfully!"),
                Err(err) => {
                    eprintln!("Failed to authenticate with Last.fm: {}", err);
                    process::exit(1);
                }
            }
            Some(scrobbler)
        }
        (None, None) => None,
        _ => {
            eprintln!("Last.fm API key or API secret are missing");
            process::exit(1);
        }
    };

    if scrobbler.is_none() && config.listenbrainz_token.is_none() {
        eprintln!("Warning: both Last.fm and ListenBrainz API credentials are missing");
    }

    mainloop::run(config, scrobbler);
}
