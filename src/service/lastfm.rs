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
use std::fs::{self, Permissions};
use std::io::{self, Write};
use std::os::unix::fs::PermissionsExt;
use std::time::Duration;

use anyhow::{Context, Result, bail};

use rustfm_scrobble_proxy::Scrobbler;

use rpassword::read_password;

use serde::Deserialize;

use crate::config::config_dir;
use crate::track::Track;

const SESSION_FILE: &str = "session";

const API_URL: &str = "https://ws.audioscrobbler.com/2.0/?format=json";

/// Authenticate with Last.fm either using an existing
/// session file or by logging in, and return the session key.
pub fn authenticate(api_key: &str, api_secret: &str) -> Result<String> {
    let mut path = config_dir()?;
    path.push(SESSION_FILE);

    if let Ok(session_key) = fs::read_to_string(&path) {
        // TODO: validate session
        return Ok(session_key);
    }

    let mut scrobbler = Scrobbler::new(api_key, api_secret);

    let mut input = String::new();

    print!(
        "Log in to Last.fm\n\
        Username: "
    );
    io::stdout().flush()?;

    io::stdin().read_line(&mut input)?;
    input.pop();
    let username = input.clone();

    input.clear();

    print!("Password: ");
    io::stdout().flush()?;

    let password = read_password().context("Failed to read password")?;

    let session_response = scrobbler.authenticate_with_password(&username, &password)?;

    let _ = fs::write(&path, &session_response.key);
    let _ = fs::set_permissions(&path, Permissions::from_mode(0o600));

    Ok(session_response.key)
}

/// An error returned by the Last.fm API.
#[derive(Deserialize)]
struct ApiError {
    error: u32,
    message: String,
}

/// A minimal Last.fm scrobbling client.
///
/// This exists instead of `rustfm_scrobble_proxy::Scrobbler` because that crate
/// has no way to submit the `albumArtist` parameter, without which Last.fm
/// assumes the album artist equals the track artist. That splits compilations
/// and DJ mixes into a separate album per track artist.
pub struct Client {
    api_key: String,
    api_secret: String,
    session_key: String,
}

impl Client {
    pub fn new(api_key: &str, api_secret: &str, session_key: &str) -> Self {
        Self {
            api_key: api_key.to_owned(),
            api_secret: api_secret.to_owned(),
            session_key: session_key.trim().to_owned(),
        }
    }

    /// Submit a "now playing" request for a track.
    ///
    /// Last.fm keeps the status up for the length of the track when it is known,
    /// so the length is submitted along with it.
    pub fn now_playing(&self, track: &Track, length: Option<Duration>) -> Result<()> {
        self.request("track.updateNowPlaying", track_params(track, length))
    }

    /// Scrobble a track that started playing at `timestamp` seconds since the
    /// UNIX epoch.
    pub fn scrobble(&self, track: &Track, timestamp: u64, length: Option<Duration>) -> Result<()> {
        let mut params = track_params(track, length);
        params.push(("timestamp", timestamp.to_string()));

        self.request("track.scrobble", params)
    }

    /// Send a signed, authenticated POST request to the Last.fm API.
    fn request(&self, method: &str, mut params: Vec<(&str, String)>) -> Result<()> {
        params.push(("method", method.to_owned()));
        params.push(("api_key", self.api_key.clone()));
        params.push(("sk", self.session_key.clone()));
        params.push(("api_sig", signature(&params, &self.api_secret)));

        let response = attohttpc::post(API_URL)
            .form(&params)
            .context("Failed to encode request")?
            .send()
            .context("Failed to send request")?;

        let status = response.status();
        let body = response.text().context("Failed to read response")?;

        if let Ok(err) = serde_json::from_str::<ApiError>(&body) {
            bail!("Last.fm API error {}: {}", err.error, err.message);
        }

        if !status.is_success() {
            bail!("Last.fm returned status {status}");
        }

        Ok(())
    }
}

/// Build the track metadata parameters shared by scrobbles and status updates.
fn track_params(track: &Track, length: Option<Duration>) -> Vec<(&'static str, String)> {
    let mut params = vec![
        ("artist", track.artist().to_owned()),
        ("track", track.title().to_owned()),
    ];

    if let Some(album) = track.album() {
        params.push(("album", album.to_owned()));
    }

    // Last.fm only wants `albumArtist` when it differs from the track artist
    if let Some(album_artist) = track.album_artist().filter(|a| *a != track.artist()) {
        params.push(("albumArtist", album_artist.to_owned()));
    }

    if let Some(length) = length {
        params.push(("duration", length.as_secs().to_string()));
    }

    params
}

/// Compute the Last.fm API signature: the parameters concatenated in order of
/// their keys, followed by the API secret, hashed with MD5.
fn signature(params: &[(&str, String)], api_secret: &str) -> String {
    let mut params: Vec<_> = params.iter().collect();
    params.sort_by_key(|(key, _)| *key);

    let mut sig = String::new();
    for (key, value) in params {
        sig.push_str(key);
        sig.push_str(value);
    }
    sig.push_str(api_secret);

    format!("{:x}", md5::compute(sig))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signature() {
        // Parameters are sorted by key, concatenated as key/value pairs and
        // suffixed with the API secret before hashing
        let params = [
            ("track", "Psycho".to_owned()),
            ("api_key", "key".to_owned()),
            ("artist", "Dimension".to_owned()),
        ];

        let expected = md5::compute("api_keykeyartistDimensiontrackPsychosecret");

        assert_eq!(signature(&params, "secret"), format!("{expected:x}"));
    }

    #[test]
    fn test_track_params() {
        // A track without an album, album artist or length only submits artist and title

        let track = Track::new("Dimension", "Psycho", None);

        assert_eq!(
            track_params(&track, None),
            vec![
                ("artist", "Dimension".to_owned()),
                ("track", "Psycho".to_owned()),
            ]
        );

        // An album artist that differs from the track artist is submitted

        let track = Track::new("The Herbaliser", "Wall Crawling", Some("The K&D Sessions"))
            .with_album_artist(Some("Kruder & Dorfmeister"));

        assert_eq!(
            track_params(&track, None),
            vec![
                ("artist", "The Herbaliser".to_owned()),
                ("track", "Wall Crawling".to_owned()),
                ("album", "The K&D Sessions".to_owned()),
                ("albumArtist", "Kruder & Dorfmeister".to_owned()),
            ]
        );

        // An album artist equal to the track artist is redundant and omitted

        let track = Track::new("Men At Work", "Down Under", Some("Business As Usual"))
            .with_album_artist(Some("Men At Work"));

        assert_eq!(
            track_params(&track, None),
            vec![
                ("artist", "Men At Work".to_owned()),
                ("track", "Down Under".to_owned()),
                ("album", "Business As Usual".to_owned()),
            ]
        );

        // A known track length is submitted as the duration, in seconds

        let track = Track::new("Dimension", "Psycho", None);

        assert_eq!(
            track_params(&track, Some(Duration::from_millis(215_500))),
            vec![
                ("artist", "Dimension".to_owned()),
                ("track", "Psycho".to_owned()),
                ("duration", "215".to_owned()),
            ]
        );
    }
}
