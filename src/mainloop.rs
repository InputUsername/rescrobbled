// Copyright (C) 2023 Koen Bolhuis
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
use std::time::{Duration, Instant, SystemTime};

use anyhow::{Context, Result, anyhow};

use mpris::{Metadata, PlaybackStatus, PlayerFinder};

use crate::config::Config;
use crate::filter::{FilterResult, filter_metadata};
use crate::player;
use crate::service::Service;
use crate::track::Track;

const POLL_INTERVAL: Duration = Duration::from_millis(500);

/// Tracks up to this length are never scrobbled.
const MIN_LENGTH: Duration = Duration::from_secs(30);

/// A track has been played enough after half its length or this, whichever
/// comes first.
const MAX_PLAY_TIME: Duration = Duration::from_secs(4 * 60);

/// How often the "now playing" status is renewed while a track keeps playing.
const NOW_PLAYING_INTERVAL: Duration = Duration::from_secs(60);

/// How far a track is allowed to play past its length before it counts as
/// having started over.
const RESTART_GRACE: Duration = Duration::from_secs(5);

/// The track playing on the active player, and how much of it has been played.
struct Playing {
    /// The track as reported by the player, used to notice when it changes.
    track: Track,
    /// The filtered track that gets submitted, or `None` if it is ignored.
    submission: Option<Track>,
    /// The length of the track, if the player reports one.
    length: Option<Duration>,
    /// The time and date the track started playing.
    start: SystemTime,
    /// How long the track has played, not counting time spent paused.
    play_time: Duration,
    /// Start of the stretch of playback not yet counted towards `play_time`.
    timer: Instant,
    /// When the "now playing" status was last updated.
    status_updated: Instant,
    /// Whether the track has been scrobbled already.
    submitted: bool,
}

impl Playing {
    /// Count the time since the previous poll as play time.
    fn tick(&mut self) {
        self.play_time += self.timer.elapsed();
        self.timer = Instant::now();
    }

    /// Skip the time since the previous poll, which was spent paused.
    fn hold(&mut self) {
        self.timer = Instant::now();
    }

    /// Whether the track played all the way through.
    ///
    /// The play time lags the position by up to one poll, so a track within a
    /// poll of its length counts as having played completely. That keeps the
    /// last track of a queue from going unscrobbled when its player pauses on
    /// the final moment instead of moving on.
    fn played_completely(&self) -> bool {
        self.length
            .is_some_and(|length| self.play_time + POLL_INTERVAL >= length)
    }

    /// Whether the track played past its length, so it must have started over.
    ///
    /// The play time is allowed to run a little past the length first, so that
    /// the last moments of a track are not taken for a repeat while the player
    /// has yet to report the next one.
    fn started_over(&self) -> bool {
        self.length
            .is_some_and(|length| self.play_time >= length + RESTART_GRACE)
    }
}

/// Decide whether a track that has been playing for `play_time` is a scrobble.
///
/// A track counts as played once it is longer than 30 seconds and has been
/// playing for at least half its length, or for 4 minutes, whichever occurs
/// earlier. The `min-play-time` config option replaces the play time that is
/// required; tracks that are too short stay out either way.
fn should_scrobble(config: &Config, length: Option<Duration>, play_time: Duration) -> bool {
    if length.is_some_and(|length| length <= MIN_LENGTH) {
        return false;
    }

    let required = config.min_play_time.unwrap_or_else(|| match length {
        Some(length) => (length / 2).min(MAX_PLAY_TIME),
        // Neither rule can be checked without a length, but a track that played
        // for 4 minutes satisfies both, however long it turns out to be
        None => MAX_PLAY_TIME,
    });

    play_time >= required
}

/// Start following a track: announce it and tell the services it is playing.
fn start_track(
    config: &Config,
    services: &[Service],
    track: Track,
    metadata: &Metadata,
    length: Option<Duration>,
) -> Playing {
    print!(
        "----\n\
        Now playing: {} - {}",
        track.artist(),
        track.title(),
    );
    if let Some(album) = track.album() {
        print!(" ({album}");
        if let Some(album_artist) = track.album_artist() {
            print!(" by {album_artist}");
        }
        print!(")");
    }
    println!();

    let submission = match filter_metadata(config, track.clone(), metadata) {
        Ok(FilterResult::Filtered(track)) | Ok(FilterResult::NotFiltered(track)) => Some(track),
        Ok(FilterResult::Ignored) => {
            println!("Track ignored");
            None
        }
        Err(err) => {
            eprintln!("{:?}", err);
            None
        }
    };

    if let Some(submission) = submission.as_ref() {
        for service in services.iter() {
            match service.now_playing(submission, length) {
                Ok(()) => println!("Status updated on {} successfully", service),
                Err(err) => eprintln!("{:?}", err),
            }
        }
    }

    let now = Instant::now();

    Playing {
        track,
        submission,
        length,
        start: SystemTime::now(),
        play_time: Duration::from_secs(0),
        timer: now,
        status_updated: now,
        submitted: false,
    }
}

/// Renew the "now playing" status of the services where it expires, so that it
/// keeps showing for as long as the track is playing.
fn refresh_now_playing(services: &[Service], playing: &mut Playing) {
    if playing.status_updated.elapsed() < NOW_PLAYING_INTERVAL {
        return;
    }

    playing.status_updated = Instant::now();

    let Some(track) = playing.submission.as_ref() else {
        return;
    };

    for service in services
        .iter()
        .filter(|service| service.now_playing_expires())
    {
        // The status was announced when the track started, so only report failures
        if let Err(err) = service.now_playing(track, playing.length) {
            eprintln!("{:?}", err);
        }
    }
}

/// Scrobble a track whose play has come to an end, timestamped with the time
/// and date it started playing.
///
/// This is where every scrobble is submitted: a track that played completely is
/// submitted the moment it did, and one whose play ended early when it is done
/// playing. Whether it counts as a scrobble at all is up to [`should_scrobble`].
fn submit_track(config: &Config, services: &[Service], playing: &mut Playing) {
    if playing.submitted || !should_scrobble(config, playing.length, playing.play_time) {
        return;
    }

    playing.submitted = true;

    let Some(track) = playing.submission.as_ref() else {
        return;
    };

    for service in services.iter() {
        match service.submit(track, playing.start, playing.length) {
            Ok(()) => println!("Track submitted to {} successfully", service),
            Err(err) => eprintln!("{:?}", err),
        }
    }
}

pub fn run(config: Config, services: Vec<Service>) -> Result<()> {
    let finder = PlayerFinder::new()
        .map_err(|err| anyhow!("{}", err))
        .context("Failed to connect to D-Bus")?;

    println!("Looking for an active MPRIS player...");

    let mut player = player::wait_for_player(&config, &finder);

    println!("Found active player {}", player.identity());

    let mut playing: Option<Playing> = None;

    loop {
        if !player.is_running() {
            // The track cannot be resumed, so its play ends here
            if let Some(mut track) = playing.take() {
                submit_track(&config, &services, &mut track);
            }

            println!(
                "----\n\
                Player {} stopped, looking for a new MPRIS player...",
                player.identity()
            );

            player = player::wait_for_player(&config, &finder);

            println!("Found active player {}", player.identity());

            continue;
        }

        let status = player
            .get_playback_status()
            .map_err(|err| anyhow!("{}", err))
            .context("Failed to retrieve playback status");

        let status = match status {
            Ok(status) => status,
            Err(err) => {
                eprintln!("{:?}", err);

                thread::sleep(POLL_INTERVAL);
                continue;
            }
        };

        if status != PlaybackStatus::Playing {
            // A paused track picks up where it left off; a stopped one does not,
            // and neither does one on a player that another player takes over from
            let takeover = player::find_active(&config, &finder)
                .filter(|other| other.bus_name() != player.bus_name());

            if status == PlaybackStatus::Stopped || takeover.is_some() {
                if let Some(mut track) = playing.take() {
                    submit_track(&config, &services, &mut track);
                }
            } else if let Some(track) = playing.as_mut() {
                track.hold();
            }

            if let Some(takeover) = takeover {
                player = takeover;

                println!(
                    "----\n\
                    Found active player {}",
                    player.identity()
                );
            }

            thread::sleep(POLL_INTERVAL);
            continue;
        }

        let metadata = player
            .get_metadata()
            .map_err(|err| anyhow!("{}", err))
            .context("Failed to get metadata");

        let metadata = match metadata {
            Ok(metadata) => metadata,
            Err(err) => {
                eprintln!("{:?}", err);

                thread::sleep(POLL_INTERVAL);
                continue;
            }
        };

        let current_track = Track::from_metadata(&metadata);

        // A player that does not know the length reports it as zero
        let length = metadata.length().filter(|length| !length.is_zero());

        let ended = match playing.as_mut() {
            // A different track means the previous one ended
            Some(track) if track.track != current_track => true,
            Some(track) => {
                track.tick();

                // A track that has played completely is a scrobble there and
                // then, however long the player stays on it
                if track.played_completely() {
                    submit_track(&config, &services, track);
                }

                // The track played past its length, so it started over
                track.started_over()
            }
            None => false,
        };

        // The play ended before the track was over; it is still a scrobble if
        // it was played long enough
        if ended && let Some(mut track) = playing.take() {
            submit_track(&config, &services, &mut track);
        }

        let playing = playing.get_or_insert_with(|| {
            start_track(&config, &services, current_track, &metadata, length)
        });

        refresh_now_playing(&services, playing);

        thread::sleep(POLL_INTERVAL);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SECOND: Duration = Duration::from_secs(1);

    fn playing(length: Option<Duration>, play_time: Duration) -> Playing {
        let now = Instant::now();

        Playing {
            track: Track::default(),
            submission: None,
            length,
            start: SystemTime::now(),
            play_time,
            timer: now,
            status_updated: now,
            submitted: false,
        }
    }

    #[test]
    fn test_played_completely() {
        let length = Duration::from_secs(3 * 60);

        // A track counts as played completely once its play time is within a
        // poll of its length, so that a player pausing on the last moment of a
        // track does not leave it unscrobbled

        assert!(!playing(Some(length), length / 2).played_completely());
        assert!(!playing(Some(length), length - POLL_INTERVAL - SECOND).played_completely());
        assert!(playing(Some(length), length - POLL_INTERVAL).played_completely());
        assert!(playing(Some(length), length).played_completely());

        // A track without a length never does, however long it plays

        assert!(!playing(None, Duration::from_secs(60 * 60)).played_completely());
    }

    #[test]
    fn test_started_over() {
        let length = Duration::from_secs(3 * 60);

        // A track that played out is not taken for a repeat right away, only
        // once it keeps playing well past its length

        assert!(!playing(Some(length), length).started_over());
        assert!(!playing(Some(length), length + RESTART_GRACE - SECOND).started_over());
        assert!(playing(Some(length), length + RESTART_GRACE).started_over());

        assert!(!playing(None, Duration::from_secs(60 * 60)).started_over());
    }

    #[test]
    fn test_should_scrobble_short_track() {
        let config = Config::default();

        // A track of 30 seconds or shorter is never a scrobble, however long it plays

        let length = Some(MIN_LENGTH);

        assert!(!should_scrobble(&config, length, MIN_LENGTH));
        assert!(!should_scrobble(&config, length, MAX_PLAY_TIME));

        // A track just over 30 seconds is, once half of it has played

        let length = Some(MIN_LENGTH + SECOND);

        assert!(!should_scrobble(&config, length, MIN_LENGTH / 2));
        assert!(should_scrobble(&config, length, MIN_LENGTH / 2 + SECOND));
    }

    #[test]
    fn test_should_scrobble_half_the_length() {
        let config = Config::default();

        // A track shorter than 8 minutes is a scrobble after half its length

        let length = Duration::from_secs(5 * 60);

        assert!(!should_scrobble(&config, Some(length), length / 2 - SECOND));
        assert!(should_scrobble(&config, Some(length), length / 2));
    }

    #[test]
    fn test_should_scrobble_four_minutes() {
        let config = Config::default();

        // A track longer than 8 minutes is a scrobble after 4 minutes, well
        // before half of it has played

        let length = Duration::from_secs(20 * 60);

        assert!(!should_scrobble(
            &config,
            Some(length),
            MAX_PLAY_TIME - SECOND
        ));
        assert!(should_scrobble(&config, Some(length), MAX_PLAY_TIME));
    }

    #[test]
    fn test_should_scrobble_without_length() {
        let config = Config::default();

        // Without a length, only the 4 minute rule can be applied

        assert!(!should_scrobble(&config, None, MAX_PLAY_TIME - SECOND));
        assert!(should_scrobble(&config, None, MAX_PLAY_TIME));
    }

    #[test]
    fn test_should_scrobble_min_play_time() {
        let config = Config {
            min_play_time: Some(Duration::from_secs(10)),
            ..Default::default()
        };

        // A configured minimum play time replaces the half/4 minute rule

        assert!(!should_scrobble(
            &config,
            Some(Duration::from_secs(5 * 60)),
            Duration::from_secs(9)
        ));
        assert!(should_scrobble(
            &config,
            Some(Duration::from_secs(5 * 60)),
            Duration::from_secs(10)
        ));

        // But tracks that are too short stay out

        assert!(!should_scrobble(
            &config,
            Some(MIN_LENGTH),
            Duration::from_secs(10)
        ));
    }
}
