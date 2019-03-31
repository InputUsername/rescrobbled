use mpris::PlayerFinder;
use rustfm_scrobble::{Scrobble, Scrobbler};
use std::fmt;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::process;
use toml::Value;

struct ApiKeys {
    api_key: String,
    api_secret: String,
}

enum ConfigError {
    Io(io::Error),
    Format(String),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ConfigError::Io(err) => write!(f, "{}", err),
            ConfigError::Format(msg) => write!(f, "{}", msg),
        }
    }
}

fn get_api_keys() -> Result<ApiKeys, ConfigError> {
    let mut file = match File::open("config.toml") {
        Ok(file) => file,
        Err(err) => return Err(ConfigError::Io(err)),
    };

    let mut buffer = String::new();

    if let Err(err) = file.read_to_string(&mut buffer) {
        return Err(ConfigError::Io(err));
    }

    let value = match buffer.parse::<Value>() {
        Ok(value) => value,
        Err(_) => return Err(ConfigError::Format("Could not parse config as TOML".to_string())),
    };

    if !value["api-key"].is_str() {
        return Err(ConfigError::Format("API key is not a string".to_string()));
    }
    if !value["api-secret"].is_str() {
        return Err(ConfigError::Format("API secret is not a string".to_string()));
    }

    let key = value["api-key"].as_str().unwrap().to_string();
    let secret = value["api-secret"].as_str().unwrap().to_string();

    Ok(ApiKeys {
        api_key: key,
        api_secret: secret,
    })
}

fn main() {
    let api_keys = match get_api_keys() {
        Ok(api_keys) => api_keys,
        Err(err) => {
            println!("{}", err);
            process::exit(1);
        },
    };

    let mut scrobbler = Scrobbler::new(api_keys.api_key, api_keys.api_secret);

    let mut input = String::new();
    std::io::stdin().read_line(&mut input)
        .expect("Could not read username");
    let username = input.trim().to_owned();

    input.clear();

    std::io::stdin().read_line(&mut input)
        .expect("Could not read password");
    let password = input.trim().to_owned();

    scrobbler.authenticate_with_password(username, password)
        .expect("Could not authenticate with Last.fm");

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
