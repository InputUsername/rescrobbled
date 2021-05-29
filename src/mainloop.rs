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

use std::thread;
use std::time::{Duration, Instant};

use anyhow::{anyhow, Context, Result};

use mpris::{PlaybackStatus, PlayerFinder};

use notify_rust::{Notification, Timeout};

use crate::config::Config;
use crate::filter::{filter_metadata, FilterResult};
use crate::player;
use crate::service::Service;
use crate::track::Track;

const POLL_INTERVAL: Duration = Duration::from_millis(500);

const MIN_LENGTH: Duration = Duration::from_secs(30);
const MIN_PLAY_TIME: Duration = Duration::from_secs(4 * 60);

const NOTIFICATION_TIMEOUT: Timeout = Timeout::Milliseconds(6000);

fn get_min_play_time(config: &Config, track_length: Duration) -> Duration {
    config.min_play_time.unwrap_or_else(|| {
        if (track_length / 2) < MIN_PLAY_TIME {
            track_length / 2
        } else {
            MIN_PLAY_TIME
        }
    })
}

pub fn run(config: Config, services: Vec<Service>) -> Result<()> {
    let finder = PlayerFinder::new()
        .map_err(|err| anyhow!("{}", err))
        .context("Failed to connect to D-Bus")?;

    println!("Looking for an active MPRIS player...");

    let mut player = player::wait_for_player(&config, &finder);

    println!("Found active player {}", player.identity());

    let mut previous_track = Track::default();

    let mut timer = Instant::now();
    let mut current_play_time = Duration::from_secs(0);
    let mut scrobbled_current_song = false;

    loop {
        if !player::is_active(&player) {
            println!(
                "----\n\
                Player {} stopped, looking for a new MPRIS player...",
                player.identity()
            );

            player = player::wait_for_player(&config, &finder);

            println!("Found active player {}", player.identity());

            previous_track.clear();

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

        let current_track = Track::from_metadata(&metadata);

        let length = match metadata.length() {
            Some(length) => length,
            None => {
                eprintln!("Failed to get track length");

                thread::sleep(POLL_INTERVAL);
                continue;
            }
        };

        if current_track == previous_track {
            if !scrobbled_current_song {
                let min_play_time = get_min_play_time(&config, length);

                if length > MIN_LENGTH && current_play_time > min_play_time {
                    match filter_metadata(&config, current_track) {
                        Ok(FilterResult::Filtered(track))
                        | Ok(FilterResult::NotFiltered(track)) => {
                            for service in services.iter() {
                                match service.submit(&track) {
                                    Ok(()) => {
                                        println!("Track submitted to {} successfully", service)
                                    }
                                    Err(err) => eprintln!("{}", err),
                                }
                            }
                        }
                        Ok(FilterResult::Ignored) => {}
                        Err(err) => eprintln!("{}", err),
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
            previous_track.clone_from(&current_track);

            timer = Instant::now();
            current_play_time = Duration::from_secs(0);
            scrobbled_current_song = false;

            if config.enable_notifications.unwrap_or(false) {
                Notification::new()
                    .summary(&current_track.title())
                    .body(&format!(
                        "{} - {}",
                        current_track.artist(),
                        current_track.album()
                    ))
                    .timeout(NOTIFICATION_TIMEOUT)
                    .show()
                    .unwrap();
            }

            println!(
                "----\n\
                Now playing: {} - {} ({})",
                current_track.artist(),
                current_track.title(),
                current_track.album()
            );

            match filter_metadata(&config, current_track) {
                Ok(FilterResult::Filtered(track)) | Ok(FilterResult::NotFiltered(track)) => {
                    for service in services.iter() {
                        match service.now_playing(&track) {
                            Ok(()) => println!("Status updated on {} successfully", service),
                            Err(err) => eprintln!("{}", err),
                        }
                    }
                }
                Ok(FilterResult::Ignored) => println!("Track ignored"),
                Err(err) => eprintln!("{}", err),
            }
        }

        thread::sleep(POLL_INTERVAL);
    }
}
