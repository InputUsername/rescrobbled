// Copyright (C) 2022 Koen Bolhuis
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

use std::fs::{self, File, OpenOptions};
use std::path::PathBuf;

use anyhow::{anyhow, Context, Result};

use csv::{Reader, Writer};

use serde::{Deserialize, Serialize};

use crate::config::Config;
use crate::service::Service;
use crate::track::Track;

const CACHE_DIR: &str = "rescrobbled";
const CACHE_FILE: &str = "cache.csv";

const HEADER: &[&str] = &["timestamp", "artist", "title", "album"];

const BATCH_SIZE: usize = 20;

#[derive(Debug, Deserialize, Serialize)]
pub struct CachedScrobble {
    timestamp: u64,
    #[serde(flatten)]
    track: Track,
}

impl CachedScrobble {
    pub fn timestamp(&self) -> u64 {
        self.timestamp
    }

    pub fn track(&self) -> &Track {
        &self.track
    }
}

pub fn cache_dir() -> Result<PathBuf> {
    let mut path =
        dirs::cache_dir().ok_or_else(|| anyhow!("User cache directory does not exist"))?;

    path.push(CACHE_DIR);

    if !path.exists() {
        fs::create_dir(&path).context("Failed to create cache directory")?;
    }

    Ok(path)
}

pub struct Cache {
    writer: Writer<File>,
}

impl Cache {
    pub fn new(config: &Config, services: &[Service]) -> Result<Option<Self>> {
        if !config.cache_scrobbles.unwrap_or(false) {
            return Ok(None);
        }

        let mut path = cache_dir()?;

        path.push(CACHE_FILE);

        if !path.exists() {
            let mut writer =
                Writer::from_path(&path).context("Failed to create scrobble cache file")?;

            writer
                .write_record(HEADER)
                .context("Failed to write scrobble cache file header")?;
        } else {
            let mut reader = Reader::from_path(&path)
                .context("Failed to open scrobble cache file for reading")?;

            let mut batch = Vec::with_capacity(BATCH_SIZE);
            let mut failed = Vec::new();

            let mut records = reader.deserialize().peekable();
            while let Some(record) = records.next() {
                let cached_scrobble: CachedScrobble = record?;
                batch.push(cached_scrobble);

                if batch.len() == BATCH_SIZE || records.peek().is_none() {
                    for service in services.iter() {
                        if let Err(err) = service.submit_cached(&batch) {
                            eprintln!("{:?}", err);
                            failed.append(&mut batch);
                            break;
                        }
                    }
                    batch.clear();
                }
            }

            if !failed.is_empty() {
                let mut writer = Writer::from_path(&path)
                    .context("Failed to open scrobble cache file for writing")?;

                writer.write_record(HEADER).context("Failed to write scrobble cache file header")?;
                for cached_scrobble in failed {
                    if let Err(err) = writer.serialize(cached_scrobble) {
                        eprintln!("{:?}", err);
                    }
                }

                eprintln!("Failed to submit some or all cached scrobbles; failures remain cached");
            } else {
                println!("Successfully submitted cached scrobbles");
            }
        }

        let cache_file = OpenOptions::new()
            .append(true)
            .open(&path)
            .context("Failed to open scrobble cache file for appending")?;

        let writer = Writer::from_writer(cache_file);

        Ok(Some(Self { writer }))
    }
}
