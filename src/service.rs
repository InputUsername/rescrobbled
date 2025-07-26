use std::{fmt::Display, time::SystemTime};

use anyhow::{bail, Result};

use crate::{
    config::{Config, ListenBrainzConfig},
    connection::{LastFMConnection, ListenBrainzConnection, ServiceConnection},
    track::Track,
};

enum ConnectionSettings {
    LastFM(Config),
    ListenBrainz(ListenBrainzConfig),
}

impl ConnectionSettings {
    /// Try to create a [`ServiceConnection`] from this settings
    fn connect(&self) -> Result<Box<dyn ServiceConnection>> {
        let connection: Box<dyn ServiceConnection> = match self {
            Self::LastFM(config) => Box::new(LastFMConnection::new(config)?),
            Self::ListenBrainz(listenbrainz_config) => {
                Box::new(ListenBrainzConnection::new(listenbrainz_config)?)
            }
        };
        println!("Authenticated with {} successfully!", connection);
        Ok(connection)
    }
}

impl Display for ConnectionSettings {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::LastFM(_) => write!(f, "last.fm")?,
            Self::ListenBrainz(config) => {
                write!(f, "ListenBrainz")?;
                if let Some(custom_url) = &config.url {
                    write!(f, " ({})", custom_url)?;
                };
            }
        }
        Ok(())
    }
}

pub struct Service {
    connection: Option<Box<dyn ServiceConnection>>,
    settings: ConnectionSettings,
}

impl Service {
    fn new(settings: ConnectionSettings) -> Self {
        Self {
            connection: None,
            settings,
        }
    }

    /// Add all services specified in the config (do not attempt to connect)
    pub fn instanciate_all(config: &Config) -> Vec<Service> {
        let mut services = Vec::new();

        if LastFMConnection::is_configured(config) {
            services.push(Service::new(ConnectionSettings::LastFM(config.clone())))
        }

        for lb in config.listenbrainz.iter().flatten() {
            services.push(Service::new(ConnectionSettings::ListenBrainz(lb.clone())))
        }

        if services.is_empty() {
            eprintln!("Warning: no scrobbling services configured");
        }

        services
    }

    pub fn get_connection(&self) -> Option<&Box<dyn ServiceConnection>> {
        self.connection.as_ref()
    }

    pub fn connect(&mut self) -> Result<()> {
        if self.connection.is_some() {
            return Ok(());
        }

        let connection = self.settings.connect()?;
        self.connection = Some(connection);
        Ok(())
    }

    /// Submit a "now playing" request.
    pub fn now_playing(&self, track: &Track) -> Result<()> {
        if let Some(connection) = self.get_connection() {
            connection.now_playing(track)
        } else {
            bail!("{} disconnected, can’t mark as \"now playing\"", self)
        }
    }

    /// Scrobble a track.
    pub fn submit(&self, track: &Track, track_start: Option<&SystemTime>) -> Result<()> {
        if let Some(connection) = self.get_connection() {
            connection.submit(track, track_start)
        } else {
            bail!("{} disconnected, can’t submit", self)
        }
    }
}

impl Display for Service {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(connection) = &self.connection {
            write!(f, "{}", connection)
        } else {
            write!(f, "{}", self.settings)
        }
    }
}
