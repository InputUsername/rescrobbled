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

use listenbrainz_rust::Listen;
use mpris::{PlaybackStatus, PlayerFinder};
use rustfm_scrobble::{Scrobble, Scrobbler};

use notify_rust::{Notification, Timeout};
use std::process;
use std::thread;
use std::time::{Duration, Instant};

use crate::config::Config;
use crate::player;

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

pub fn run(config: Config, scrobbler: Option<Scrobbler>) {
    let finder = match PlayerFinder::new() {
        Ok(finder) => finder,
        Err(err) => {
            eprintln!("Failed to connect to D-Bus: {}", err);
            process::exit(1);
        }
    };

    println!("Looking for an active MPRIS player...");

    let mut player = player::wait_for_player(&config, &finder);

    println!("Found active player {}", player.identity());

    let mut previous_artist = String::new();
    let mut previous_title = String::new();
    let mut previous_album = String::new();

    let mut timer = Instant::now();
    let mut current_play_time = Duration::from_secs(0);
    let mut scrobbled_current_song = false;

    loop {
        if !player::is_active(&player) {
            println!(
                "Player {} stopped, looking for a new MPRIS player...",
                player.identity()
            );

            player = player::wait_for_player(&config, &finder);

            println!("Found active player {}", player.identity());

            previous_artist.clear();
            previous_title.clear();
            previous_album.clear();

            timer = Instant::now();
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
            .map(|&artist| artist)
            .unwrap_or("");

        let title = metadata.title().unwrap_or("");

        let album = metadata.album_name().unwrap_or("");

        let length = match metadata.length() {
            Some(length) => length,
            None => {
                eprintln!("Failed to get track length");

                thread::sleep(POLL_INTERVAL);
                continue;
            }
        };

        if artist == previous_artist && title == previous_title && album == previous_album {
            if !scrobbled_current_song {
                let min_play_time = get_min_play_time(length, &config);

                if length > MIN_LENGTH && current_play_time > min_play_time {
                    let scrobble = Scrobble::new(artist, title, album);

                    if let Some(ref scrobbler) = scrobbler {
                        match scrobbler.scrobble(&scrobble) {
                            Ok(_) => println!("Track submitted to Last.fm successfully"),
                            Err(err) => eprintln!("Failed to submit track to Last.fm: {}", err),
                        }
                    }

                    if let Some(ref token) = config.listenbrainz_token {
                        let listen = Listen {
                            artist: artist,
                            track: title,
                            album: album,
                        };
                        match listen.single(token) {
                            Ok(_) => println!("Track submitted to ListenBrainz successfully"),
                            Err(err) => {
                                eprintln!("Failed to submit track to ListenBrainz: {}", err)
                            }
                        }
                    }
                    scrobbled_current_song = true;
                }
            } else if current_play_time >= length {
                current_play_time = Duration::from_secs(0);
                scrobbled_current_song = false;
            }

            current_play_time += timer.elapsed();
            timer = Instant::now();
        } else {
            previous_artist.clear();
            previous_artist.push_str(artist);
            previous_title.clear();
            previous_title.push_str(title);
            previous_album.clear();
            previous_album.push_str(album);

            timer = Instant::now();
            current_play_time = Duration::from_secs(0);
            scrobbled_current_song = false;

            println!("----");
            println!("Now playing: {} - {} ({})", artist, title, album);

            if config.enable_notifications.unwrap_or(false) {
                Notification::new()
                    .summary(&title)
                    .body(&format!("{} - {}", artist, album))
                    .timeout(Timeout::Milliseconds(6000))
                    .show()
                    .unwrap();
            }

            let scrobble = Scrobble::new(artist, title, album);

            if let Some(ref scrobbler) = scrobbler {
                match scrobbler.now_playing(&scrobble) {
                    Ok(_) => println!("Status updated on Last.fm successfully"),
                    Err(err) => eprintln!("Failed to update status on Last.fm: {}", err),
                }
            }

            if let Some(ref token) = config.listenbrainz_token {
                let listen = Listen {
                    artist: artist,
                    track: title,
                    album: album,
                };
                match listen.playing_now(token) {
                    Ok(_) => println!("Status updated on ListenBrainz successfully"),
                    Err(err) => eprintln!("Failed to update status on ListenBrainz: {}", err),
                }
            }
        }

        thread::sleep(POLL_INTERVAL);
    }
}
