use mpris::{PlayerFinder, PlaybackStatus};
use rustfm_scrobble::{Scrobble, Scrobbler};

use std::process;
use std::thread;
use std::fs;
use std::io;
use std::io::Write;
use std::time::Duration;

mod config;

const SESSION_FILE: &str = ".session";

fn main() {
    let api_keys = match config::load_config() {
        Ok(api_keys) => api_keys,
        Err(err) => {
            println!("Error while loading config: {}", err);
            process::exit(1);
        },
    };

    let mut scrobbler = Scrobbler::new(api_keys.api_key, api_keys.api_secret);

    if let Ok(session_key) = fs::read_to_string(SESSION_FILE) {
        // TODO: validate session
        scrobbler.authenticate_with_session_key(session_key);
    } else {
        let mut input = String::new();

        print!("Username: ");
        io::stdout().flush().unwrap();
        
        io::stdin().read_line(&mut input)
            .expect("Could not read username");
        input.pop();
        let username = input.clone();

        input.clear();

        print!("Password: ");
        io::stdout().flush().unwrap();

        io::stdin().read_line(&mut input)
            .expect("Could not read password");
        input.pop();
        let password = input.clone();

        let session_response = match scrobbler.authenticate_with_password(username, password) {
            Ok(res) => res,
            Err(err) => {
                println!("Error authenticating with Last.fm: {}", err);
                process::exit(1);
            },
        };

        // We don't care whether storing the session works;
        // it's simply convenient if it does
        let _ = fs::write(SESSION_FILE, session_response.key);
    }

    let finder = match PlayerFinder::new() {
        Ok(finder) => finder,
        Err(_) => {
            println!("Could not connect to D-Bus");
            process::exit(1);
        },
    };

    let player = match finder.find_active() {
        Ok(player) => player,
        Err(_) => {
            println!("Could not find any active players");
            process::exit(1);
        }
    };

    let mut previous_track = String::new();

    loop {
        thread::sleep(Duration::from_secs(1));

        if !player.is_running() {
            break;
        }

        let status = match player.get_playback_status() {
            Ok(status) => status,
            Err(err) => {
                println!("Error retrieving playback status: {}", err);
                process::exit(1);
            },
        };

        if status != PlaybackStatus::Playing {
            previous_track.clear();
            continue;
        }

        let meta = match player.get_metadata() {
            Ok(meta) => meta,
            Err(err) => {
                println!("Error retrieving track metadata: {}", err);
                process::exit(1);
            },
        };

        let track_id = meta.track_id();
        
        // Behavior will change in mpris-rs 2.0.0:
        // track_id will be an Option, which will be None if no track found
        if track_id.is_empty() {
            continue;
        }

        if track_id == previous_track {
            continue;
        }

        previous_track.clear();
        previous_track.push_str(track_id);

        let artist = meta.artists()
            .and_then(|artists| artists.first())
            .map(|artist| artist.clone())
            .unwrap_or_else(|| String::new());
        
        let title = meta.title()
            .map(|title| title.to_owned())
            .unwrap_or_else(|| String::new());

        let album = meta.album_name()
            .map(|album| album.to_owned())
            .unwrap_or_else(|| String::new());

        println!("Now playing: {} - {}\n{}", artist, title, album);
        
        let scrobble = Scrobble::new(artist, title, album);
        let np_result = scrobbler.now_playing(scrobble)
            .unwrap();

        println!("Scrobble result: {:#?}", np_result);

        // TODO: scrobble as well in addition to now playing
    }
}
