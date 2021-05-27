// Rescrobbled is an MPRIS music scrobbler daemon.
//
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

use anyhow::Result;

mod config;

use config::load_config;

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() -> Result<()> {
    if std::env::args().any(|arg| arg == "-v" || arg == "--version") {
        println!("rescrobbled v{}", VERSION);
        return Ok(());
    }

    let _config = load_config()?;

    Ok(())
}
