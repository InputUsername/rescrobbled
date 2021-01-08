// Copyright (C) 2019 Koen Bolhuis
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

use std::fs;
use std::io;
use std::io::Write;

use crate::config;

use rustfm_scrobble::{Scrobbler, ScrobblerError};

const SESSION_FILE: &str = "session";

pub fn authenticate(scrobbler: &mut Scrobbler) -> Result<(), ScrobblerError> {
    let mut path = config::config_dir().unwrap();
    path.push(SESSION_FILE);

    if let Ok(session_key) = fs::read_to_string(&path) {
        // TODO: validate session
        scrobbler.authenticate_with_session_key(&session_key);
    } else {
        let mut input = String::new();

        print!("Username: ");
        io::stdout().flush().unwrap();

        io::stdin().read_line(&mut input).unwrap();
        input.pop();
        let username = input.clone();

        input.clear();

        print!("Password: ");
        io::stdout().flush().unwrap();

        io::stdin().read_line(&mut input).unwrap();
        input.pop();
        let password = input;

        let session_response = scrobbler.authenticate_with_password(&username, &password)?;

        // We don't care whether storing the session works;
        // it's simply convenient if it does
        let _ = fs::write(path, session_response.key);
    }

    Ok(())
}
