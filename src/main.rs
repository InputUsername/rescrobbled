use rustfm_scrobble::Scrobbler;

use std::process;

mod config;
mod auth;
mod mainloop;

fn main() {
    let config = match config::load_config() {
        Ok(config) => config,
        Err(err) => {
            eprintln!("Error while loading config: {}", err);
            process::exit(1);
        },
    };

    let mut scrobbler = Scrobbler::new(config.api_key, config.api_secret);

    match auth::authenticate(&mut scrobbler) {
        Ok(_) => println!("Authenticated with Last.fm successfully!"),
        Err(err) => {
            eprintln!("Failed to authenticate with Last.fm: {}", err);
            process::exit(1);
        },
    }

    mainloop::run(&scrobbler);
}