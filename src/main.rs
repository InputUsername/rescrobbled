// Rescrobbled is an MPRIS music scrobbler daemon.
//
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

use std::env;
use std::process::ExitCode;

use anyhow::Result;

use tracing::{error, info};

mod config;
mod filter;
mod mainloop;
mod player;
mod service;
mod track;

use config::load_config;
use service::Service;

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn run() -> Result<()> {
    let config = load_config()?;
    let services = Service::initialize_all(&config);

    mainloop::run(config, services)
}

fn main() -> ExitCode {
    if env::args().any(|arg| arg == "-v" || arg == "--version") {
        println!("rescrobbled v{VERSION}");
        return ExitCode::SUCCESS;
    }

    tracing_subscriber::fmt().init();

    info!("rescrobbled v{VERSION}");

    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            error!("{}", err);
            ExitCode::FAILURE
        }
    }
}
