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

use std::fs::{self, Permissions};
use std::os::unix::fs::PermissionsExt;
use std::io::{self, Write};

use anyhow::Result;

use rustfm_scrobble::Scrobbler;

use crate::config::config_dir;

const SESSION_FILE: &str = "session";

/// Authenticate with Last.fm either using an existing
/// session file or by logging in.
pub fn authenticate(scrobbler: &mut Scrobbler) -> Result<()> {
    let mut path = config_dir()?;
    path.push(SESSION_FILE);

    if let Ok(session_key) = fs::read_to_string(&path) {
        // TODO: validate session
        scrobbler.authenticate_with_session_key(&session_key);
    } else {
        let mut input = String::new();

        print!(
            "Log in to Last.fm\n\
            Username: "
        );
        io::stdout().flush()?;

        io::stdin().read_line(&mut input)?;
        input.pop();
        let username = input.clone();

        input.clear();

        print!("Password: ");
        io::stdout().flush()?;

        io::stdin().read_line(&mut input)?;
        input.pop();
        let password = input;

        let session_response = scrobbler.authenticate_with_password(&username, &password)?;

        let _ = fs::write(&path, session_response.key);
        let _ = fs::set_permissions(&path, Permissions::from_mode(0o600));
    }

    Ok(())
}
