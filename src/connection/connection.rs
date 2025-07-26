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

use std::fmt::Display;
use std::time::SystemTime;

use anyhow::Result;

use crate::track::Track;

pub trait ServiceConnection: Display {
    /// Submit a "now playing" request.
    fn now_playing(&self, track: &Track) -> Result<()>;
    /// Scrobble a track.
    fn submit(&self, track: &Track, track_start: Option<&SystemTime>) -> Result<()>;
}
