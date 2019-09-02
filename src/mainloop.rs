use mpris::{PlaybackStatus, Player, PlayerFinder};
use rustfm_scrobble::{Scrobble, Scrobbler};

use std::process;
use std::thread;
use std::time::Duration;

use crate::config::Config;

const INIT_WAIT_TIME: Duration = Duration::from_secs(1);
const POLL_INTERVAL: Duration = Duration::from_millis(500);

const MIN_LENGTH: Duration = Duration::from_secs(30);
const MIN_PLAY_TIME: Duration = Duration::from_secs(4 * 60);

fn get_min_play_time(track_length: Duration, config: &Config) -> Duration {
    config.min_play_time.unwrap_or_else(|| {
        if (track_length / 2) < MIN_PLAY_TIME {
            track_length / 2
        } else {
            MIN_PLAY_TIME
        }
    })
}

fn wait_for_player(finder: &PlayerFinder) -> Player {
    loop {
        match finder.find_active() {
            Ok(player) => return player,
            Err(_) => {}
        }

        thread::sleep(INIT_WAIT_TIME);
    }
}

pub fn run(config: &Config, scrobbler: &Scrobbler) {
    let finder = match PlayerFinder::new() {
        Ok(finder) => finder,
        Err(err) => {
            eprintln!("Failed to connect to D-Bus: {}", err);
            process::exit(1);
        }
    };

    println!("Looking for an active MPRIS player...");

    let mut player = wait_for_player(&finder);

    println!("Found active player {}", player.identity());

    let mut previous_artist = String::new();
    let mut previous_title = String::new();
    let mut previous_album = String::new();

    let mut current_play_time = Duration::from_secs(0);
    let mut scrobbled_current_song = false;

    loop {
        if !player.is_running() {
            println!(
                "Player {} stopped, looking for a new MPRIS player...",
                player.identity()
            );

            player = wait_for_player(&finder);

            println!("Found active player {}", player.identity());

            previous_artist.clear();
            previous_title.clear();
            previous_album.clear();

            current_play_time = Duration::from_secs(0);
            scrobbled_current_song = false;
        }

        match player.get_playback_status() {
            Ok(PlaybackStatus::Playing) => {}
            Ok(_) => {
                thread::sleep(POLL_INTERVAL);
                continue;
            }
            Err(err) => {
                eprintln!("Failed to retrieve playback status: {}", err);

                thread::sleep(POLL_INTERVAL);
                continue;
            }
        }

        let metadata = match player.get_metadata() {
            Ok(metadata) => metadata,
            Err(err) => {
                eprintln!("Failed to get metadata: {}", err);

                thread::sleep(POLL_INTERVAL);
                continue;
            }
        };

        let artist = metadata
            .artists()
            .as_ref()
            .and_then(|artists| artists.first())
            .map(|&artist| artist.to_owned())
            .unwrap_or_else(|| String::new());

        let title = metadata
            .title()
            .map(|title| title.to_owned())
            .unwrap_or_else(|| String::new());

        let album = metadata
            .album_name()
            .map(|title| title.to_owned())
            .unwrap_or_else(|| String::new());

        if artist == previous_artist && title == previous_title && album == previous_album {
            if !scrobbled_current_song {
                let length = match metadata.length() {
                    Some(length) => length,
                    None => {
                        eprintln!("Failed to get track length");

                        thread::sleep(POLL_INTERVAL);
                        continue;
                    }
                };

                let min_play_time = get_min_play_time(length, config);

                if length > MIN_LENGTH && current_play_time > min_play_time {
                    let scrobble = Scrobble::new(artist, title, album);

                    match scrobbler.scrobble(scrobble) {
                        Ok(_) => println!("Track scrobbled successfully"),
                        Err(err) => eprintln!("Failed to scrobble song: {}", err),
                    }

                    scrobbled_current_song = true;
                }

                current_play_time += POLL_INTERVAL;
            }
        } else {
            previous_artist.clone_from(&artist);
            previous_title.clone_from(&title);
            previous_album.clone_from(&album);

            current_play_time = Duration::from_secs(0);
            scrobbled_current_song = false;

            println!("----");
            println!("Now playing: {} - {} ({})", artist, title, album);

            let scrobble = Scrobble::new(artist, title, album);

            match scrobbler.now_playing(scrobble) {
                Ok(_) => println!("Status updated successfully"),
                Err(err) => eprintln!("Failed to update status: {}", err),
            }
        }

        thread::sleep(POLL_INTERVAL);
    }
}
