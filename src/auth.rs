use std::fs;
use std::io;
use std::io::Write;

use rustfm_scrobble::{Scrobbler, ScrobblerError};

const SESSION_FILE: &str = ".session";

pub fn authenticate(scrobbler: &mut Scrobbler) -> Result<(), ScrobblerError> {
    if let Ok(session_key) = fs::read_to_string(SESSION_FILE) {
        // TODO: validate session
        scrobbler.authenticate_with_session_key(session_key);
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
        let password = input.clone();

        let session_response = scrobbler.authenticate_with_password(username, password)?;

        // We don't care whether storing the session works;
        // it's simply convenient if it does
        let _ = fs::write(SESSION_FILE, session_response.key);
    }

    Ok(())
}