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

use mpris::Metadata;

#[derive(Debug, Default, PartialEq)]
pub struct Track {
    artist: String,
    title: String,
    album: Option<String>,
}

impl Track {
    pub fn artist(&self) -> &str {
        &self.artist
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn album(&self) -> Option<&str> {
        self.album.as_deref()
    }

    pub fn new(artist: &str, title: &str, album: Option<&str>) -> Self {
        Self {
            artist: artist.to_owned(),
            title: title.to_owned(),
            album: album.and_then(|album| {
                if !album.is_empty() {
                    Some(album.to_owned())
                } else {
                    None
                }
            }),
        }
    }

    pub fn clear(&mut self) {
        self.artist.clear();
        self.title.clear();
        self.album.take();
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

        let album = metadata.album_name().and_then(|album| {
            if !album.is_empty() {
                eprintln!("album <{album}> not empty");
                Some(album.to_owned())
            } else {
                None
            }
        });

        Self {
            artist,
            title,
            album,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mpris::MetadataValue;
    use std::collections::HashMap;

    #[test]
    fn test_new() {
        // Constructing a track with an empty album should result in `None` for `Track::album()`

        assert_eq!(
            Track::new("Enter Shikari", "Live Outside", None).album(),
            None
        );

        // Constructing a track with a nonempty album should result in `Some` for `Track::album()`

        assert_eq!(
            Track::new("Dimension", "Psycho", Some("Organ")).album(),
            Some("Organ")
        );
    }

    #[test]
    fn test_from_metadata() {
        // Metadata without an album should result in a `None` for `Track::album()`

        let mut metadata_without_album = HashMap::new();
        metadata_without_album.insert(
            "xesam:artists".to_owned(),
            MetadataValue::Array(vec![MetadataValue::String("Billy Joel".to_owned())]),
        );
        metadata_without_album.insert(
            "xesam:title".to_owned(),
            MetadataValue::String("We didn't start the fire".to_owned()),
        );
        let metadata_without_album = Metadata::from(metadata_without_album);
        let track_without_album = Track::from_metadata(&metadata_without_album);

        assert_eq!(track_without_album.album(), None);

        // Metadata with an empty album should result in a `None` for `Track::album()`

        let mut metadata_empty_album = HashMap::new();
        metadata_empty_album.insert(
            "xesam:artist".to_owned(),
            MetadataValue::Array(vec![MetadataValue::String("The Prodigy".to_owned())]),
        );
        metadata_empty_album.insert(
            "xesam:title".to_owned(),
            MetadataValue::String("Wild Frontier".to_owned()),
        );
        metadata_empty_album.insert(
            "xesam:album".to_owned(),
            MetadataValue::String("".to_owned()),
        );
        let metadata_empty_album = Metadata::from(metadata_empty_album);
        let track_empty_album = Track::from_metadata(&metadata_empty_album);

        assert_eq!(track_empty_album.album(), None);

        // Metadata with a nonempty album should result in a `Some` for `Track::album()`

        let mut metadata_with_album = HashMap::new();
        metadata_with_album.insert(
            "xesam:artist".to_owned(),
            MetadataValue::Array(vec![MetadataValue::String("Men At Work".to_owned())]),
        );
        metadata_with_album.insert(
            "xesam:title".to_owned(),
            MetadataValue::String("Who Can It Be Now?".to_owned()),
        );
        metadata_with_album.insert(
            "xesam:album".to_owned(),
            MetadataValue::String("Business As Usual".to_owned()),
        );
        let metadata_with_album = Metadata::from(metadata_with_album);
        let track_with_album = Track::from_metadata(&metadata_with_album);

        assert_eq!(track_with_album.album(), Some("Business As Usual"));
    }
}
