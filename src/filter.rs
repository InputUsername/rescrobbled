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

use std::fmt::Write as _;
use std::io::Write as _;
use std::process::{Command, Stdio};

use anyhow::{Context, Result, anyhow, bail};

use mpris::Metadata;

use crate::config::Config;
use crate::track::Track;

#[derive(Debug, PartialEq)]
pub enum FilterResult {
    Filtered(Track),
    NotFiltered(Track),
    Ignored,
}

pub fn filter_metadata(config: &Config, track: Track, metadata: &Metadata) -> Result<FilterResult> {
    if config.filter_script.is_none() {
        return Ok(FilterResult::NotFiltered(track));
    }

    let path = config.filter_script.as_ref().unwrap();

    let mut child = Command::new(config.filter_script.as_ref().unwrap())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .with_context(|| format!("Failed to run filter script at {}", path.display()))?;

    let mut stdin = child
        .stdin
        .take()
        .ok_or_else(|| anyhow!("Failed to get an stdin handle for the filter script"))?;

    // Write metadata to filter script stdin

    let genre = metadata
        .get("xesam:genre")
        .and_then(|value| value.as_str_array())
        .unwrap_or_default();

    let buffer = format!(
        "{}\n{}\n{}\n{}\n",
        track.artist(),
        track.title(),
        track.album().unwrap_or(""),
        genre.join(","),
    );
    stdin
        .write_all(buffer.as_bytes())
        .context("Failed to write track metadata to filter script stdin")?;

    // Close child's stdin to prevent endless waiting
    drop(stdin);

    let output = child
        .wait_with_output()
        .context("Failed to retrieve output from filter script")?;

    if !output.status.success() {
        let mut message = "Filter script returned unsuccessully ".to_owned();
        if let Some(status) = output.status.code() {
            writeln!(message, "with status: {status}").unwrap();
        } else {
            message += "without status\n";
        }

        match String::from_utf8(output.stderr) {
            Ok(output) => write!(message, "Stderr: {output}").unwrap(),
            Err(err) => write!(message, "Stderr is not valid UTF-8: {err}").unwrap(),
        }

        bail!(message);
    }

    let output =
        String::from_utf8(output.stdout).context("Filter script stdout is not valid UTF-8")?;

    let mut output = output.split('\n');
    match (output.next(), output.next(), output.next()) {
        (Some(artist), Some(title), album) => {
            Ok(FilterResult::Filtered(Track::new(artist, title, album)))
        }
        _ => Ok(FilterResult::Ignored),
    }
}

/// Distinctive phrases used by browsers (e.g. Firefox) as the title of the
/// placeholder metadata they expose while playing media in a private/incognito
/// window, in various locales. The real track info is hidden in that case.
const PRIVATE_BROWSING_PLACEHOLDER_PHRASES: &[&str] = &[
    "is playing media",           // en
    "sedang memutar media",       // id
    "spielt medien ab",           // de
    "lit un contenu multimédia",  // fr
    "está reproduzindo mídia",    // pt-BR
    "воспроизводит медиа",        // ru
    "がメディアを再生しています", // ja
    "正在播放媒体",               // zh-CN
    "speelt media af",            // nl
    "odtwarza multimedia",        // pl
];

/// Determine whether the given metadata is a browser's private-browsing
/// placeholder, i.e. no real track information is exposed because the media is
/// playing in a private/incognito window.
///
/// Browsers hide the real track info in that case: there is no URL, no artist,
/// no album and the title is just a generic "{browser} is playing media"
/// string. Such tracks should never be scrobbled, since doing so would leak
/// that private listening happened.
pub fn is_private_browsing_placeholder(metadata: &Metadata, track: &Track) -> bool {
    let has_url = metadata.url().map(|url| !url.is_empty()).unwrap_or(false);

    let title = track.title().to_lowercase();

    track.artist().is_empty()
        && track.album().is_none()
        && !has_url
        && PRIVATE_BROWSING_PLACEHOLDER_PHRASES
            .iter()
            .any(|phrase| title.contains(phrase))
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::os::unix::fs::PermissionsExt;
    use std::path::Path;

    use super::*;

    fn write_test_script(path: &Path, contents: &str) {
        fs::write(path, contents).unwrap();
        fs::set_permissions(path, fs::Permissions::from_mode(0o755)).unwrap();
    }

    #[test]
    fn test_filter_script() {
        let mut config = Config::default();
        let temp_dir = tempfile::tempdir().unwrap();

        let path = temp_dir.path().join("filter.sh");
        const FILTER_SCRIPT: &str = "#!/usr/bin/env sh
read artist
read title
read album
echo \"Artist=$artist\"
echo \"Title=$title\"
echo \"Album=$album\"
";

        write_test_script(&path, FILTER_SCRIPT);

        config.filter_script = Some(path);

        assert_eq!(
            filter_metadata(
                &config,
                Track::new("lorem", "ipsum", Some("dolor")),
                &Metadata::new("track_id"),
            )
            .unwrap(),
            FilterResult::Filtered(Track::new(
                "Artist=lorem",
                "Title=ipsum",
                Some("Album=dolor")
            ))
        );

        // Script that produces no output should result in `FilterResult::Ignored`

        let path_ignore = temp_dir.path().join("filter_ignore.sh");
        const FILTER_SCRIPT_IGNORE: &str = "#!/usr/bin/env sh
true
";

        write_test_script(&path_ignore, FILTER_SCRIPT_IGNORE);

        config.filter_script = Some(path_ignore);

        assert_eq!(
            filter_metadata(
                &config,
                Track::new("lorem", "ipsum", Some("dolor")),
                &Metadata::new("track_id"),
            )
            .unwrap(),
            FilterResult::Ignored
        );

        // Not using a filter script should result in `FilterResult::NotFiltered`

        config.filter_script = None;

        assert_eq!(
            filter_metadata(
                &config,
                Track::new("lorem", "ipsum", Some("dolor")),
                &Metadata::new("track_id"),
            )
            .unwrap(),
            FilterResult::NotFiltered(Track::new("lorem", "ipsum", Some("dolor")))
        );

        // Album should be optional, empty album should still result in `FilterResult::Filtered`

        let path_no_album = temp_dir.path().join("filter_no_album.sh");
        const FILTER_SCRIPT_NO_ALBUM: &str = "#!/usr/bin/env sh
read artist
read title
read album
read genre
echo \"$artist\"
echo \"$title\"
echo \"$album\"
";

        write_test_script(&path_no_album, FILTER_SCRIPT_NO_ALBUM);

        config.filter_script = Some(path_no_album);

        assert_eq!(
            filter_metadata(
                &config,
                Track::new("lorem", "ipsum", None),
                &Metadata::new("track_id"),
            )
            .unwrap(),
            FilterResult::Filtered(Track::new("lorem", "ipsum", None)),
        )
    }

    fn metadata_with(
        title: &str,
        artist: Option<&str>,
        album: Option<&str>,
        url: Option<&str>,
    ) -> Metadata {
        let mut map = std::collections::HashMap::new();
        map.insert(
            "xesam:title".to_owned(),
            mpris::MetadataValue::String(title.to_owned()),
        );
        if let Some(artist) = artist {
            map.insert(
                "xesam:artist".to_owned(),
                mpris::MetadataValue::Array(vec![mpris::MetadataValue::String(artist.to_owned())]),
            );
        }
        if let Some(album) = album {
            map.insert(
                "xesam:album".to_owned(),
                mpris::MetadataValue::String(album.to_owned()),
            );
        }
        if let Some(url) = url {
            map.insert(
                "xesam:url".to_owned(),
                mpris::MetadataValue::String(url.to_owned()),
            );
        }
        Metadata::from(map)
    }

    #[test]
    fn test_private_browsing_placeholder() {
        // Firefox hides real metadata in a private window: generic title,
        // empty artist, no album and no URL.

        let metadata = metadata_with("Firefox is playing media", Some(""), None, None);
        let track = Track::from_metadata(&metadata);
        assert!(is_private_browsing_placeholder(&metadata, &track));

        // Localized placeholder should also be detected

        let metadata_id = metadata_with(
            "Firefox Developer Edition sedang memutar media",
            None,
            None,
            None,
        );
        let track_id = Track::from_metadata(&metadata_id);
        assert!(is_private_browsing_placeholder(&metadata_id, &track_id));

        // A real track playing in a normal window has a URL, so it is not a
        // private-browsing placeholder even if the title coincidentally
        // contains the phrase.

        let metadata_real = metadata_with(
            "Some band is playing media",
            Some("The Band"),
            Some("Album"),
            Some("https://example.com/watch?v=123"),
        );
        let track_real = Track::from_metadata(&metadata_real);
        assert!(!is_private_browsing_placeholder(
            &metadata_real,
            &track_real
        ));

        // A real track without a URL (e.g. an internet radio stream) but with
        // a real title that does not match any placeholder phrase is not
        // ignored.

        let metadata_radio = metadata_with("Some song title", Some(""), None, None);
        let track_radio = Track::from_metadata(&metadata_radio);
        assert!(!is_private_browsing_placeholder(
            &metadata_radio,
            &track_radio
        ));
    }
}
