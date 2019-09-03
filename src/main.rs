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

use rustfm_scrobble::Scrobbler;

use std::process;

mod auth;
mod config;
mod mainloop;

fn main() {
    let config = match config::load_config() {
        Ok(config) => config,
        Err(err) => {
            eprintln!("Error while loading config: {}", err);
            eprintln!("\t$HOME/.config/rescrobbled/config.toml must be formatted as follows:");
            eprintln!("\tapi-key = \"apikeystring\"");
            eprintln!("\tapi-secret = \"apisecretstring\"");
            eprintln!("\tlb-token = \"tokenuuid\"");
            eprintln!("\tenable-notifications = true");
            process::exit(1);
        }
    };

    let mut scrobbler = Scrobbler::new(config.api_key.clone(), config.api_secret.clone());

    match auth::authenticate(&mut scrobbler) {
        Ok(_) => println!("Authenticated with Last.fm successfully!"),
        Err(err) => {
            eprintln!("Failed to authenticate with Last.fm: {}", err);
            process::exit(1);
        }
    }

    mainloop::run(&config, &scrobbler);
}
