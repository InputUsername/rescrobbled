use mpris::PlayerFinder;
use rustfm_scrobble::{Scrobble, Scrobbler};

use std::process;
use std::fs;
use std::io;
use std::io::Write;

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
        let username = input.trim().to_owned();

        input.clear();

        print!("Password: ");
        io::stdout().flush().unwrap();

        io::stdin().read_line(&mut input)
            .expect("Could not read password");
        let password = input.trim().to_owned();

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

    let player = PlayerFinder::new()
        .expect("Could not connect to D-Bus")
        .find_active()
        .expect("Could not find any player");

    let meta = player.get_metadata()
        .expect("Could not get metadata");

    let length = meta.length_in_microseconds()
        .expect("No length in microseconds found for this track");

    if length < 30 * 1000 * 1000 {
        println!("Not allowed to scrobble this track :(");
        return;
    }

    let artist = meta.artists()
        .expect("No artist list found for this track")
        .first()
        .expect("No artist found for this track");
    let title = meta.title()
        .expect("No title found for this track");
    let album = meta.album_name()
        .expect("No album name found for this track");

    println!("Now playing: {} - {} ({})", artist, title, album);

    let scrobble = Scrobble::new(artist.clone(), title.to_owned(), album.to_owned());
    let np_result = scrobbler.now_playing(scrobble.clone())
        .unwrap();
    let scrobble_result = scrobbler.scrobble(scrobble)
        .unwrap();

    println!("Now playing result:\n{:#?}", np_result);
    println!("Scrobble result:\n{:#?}", scrobble_result);
}
