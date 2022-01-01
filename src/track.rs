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

use mpris::Metadata;

use rustfm_scrobble::Scrobble;

#[derive(Debug, Default, PartialEq)]
pub struct Track {
    artist: String,
    title: String,
    album: String,
}

impl Track {
    pub fn artist(&self) -> &str {
        &self.artist
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn album(&self) -> &str {
        &self.album
    }

    pub fn new(artist: &str, title: &str, album: &str) -> Self {
        Self {
            artist: artist.to_owned(),
            title: title.to_owned(),
            album: album.to_owned(),
        }
    }

    pub fn clear(&mut self) {
        self.artist.clear();
        self.title.clear();
        self.album.clear();
    }

    pub fn clone_from(&mut self, other: &Self) {
        self.artist.clone_from(&other.artist);
        self.title.clone_from(&other.title);
        self.album.clone_from(&other.album);
    }

    pub fn from_metadata(metadata: &Metadata) -> Self {
        let artist = metadata
            .artists()
            .as_ref()
            .and_then(|artists| artists.first().copied())
            .unwrap_or("")
            .to_owned();

        let title = metadata.title().unwrap_or("").to_owned();

        let album = metadata.album_name().unwrap_or("").to_owned();

        Self {
            artist,
            title,
            album,
        }
    }
}

impl From<&Track> for Scrobble {
    fn from(track: &Track) -> Scrobble {
        Scrobble::new(track.artist(), track.title(), track.album())
    }
}
