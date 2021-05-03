use mpris::Metadata;

#[derive(Debug, Default, PartialEq)]
pub struct Track {
    artist: String,
    title: String,
    album: String,
}

impl Track {
    pub fn artist(&self) -> &str { &self.artist }
    pub fn title(&self) -> &str { &self.title }
    pub fn album(&self) -> &str { &self.album }

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
        let artist = metadata.artists()
            .as_ref()
            .and_then(|artists| artists.first())
            .map(|&artist| artist)
            .unwrap_or("")
            .to_owned();

        let title = metadata.title().unwrap_or("").to_owned();

        let album = metadata.album_name().unwrap_or("").to_owned();

        Self { artist, title, album }
    }
}
