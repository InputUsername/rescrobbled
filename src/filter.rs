use std::process::{Command, Stdio};
use std::io::Write;

use crate::config::Config;

pub fn filter_metadata(config: &Config, artist: &str, title: &str, album: &str) -> Option<(String, String, String)> {
    if config.filter_script.is_none() {
        return None;
    }

    let child = Command::new(config.filter_script.as_ref().unwrap())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn();

    let mut child = match child {
        Ok(child) => child,
        Err(err) => {
            eprintln!("Failed to run filter script: {}", err);
            return None;
        }
    };

    let stdin = child.stdin.take();
    let mut stdin = if let Some(stdin) = stdin {
        stdin
    } else {
        eprintln!("Failed to get a stdin handle for the filter script");
        return None;
    };

    let buffer = format!("{}\n{}\n{}\n", artist, title, album);
    if let Err(err) = stdin.write_all(buffer.as_bytes()) {
        eprintln!("Failed to write metadata to filter script stdin: {}", err);
        return None;
    }

    // Close child's stdin to prevent endless waiting
    drop(stdin);

    let output = match child.wait_with_output() {
        Ok(output) => output,
        Err(err) => {
            eprintln!("Failed to retrieve output from filter script: {}", err);
            return None;
        }
    };

    if !output.status.success() {
        eprint!("Filter script returned unsuccessfully ");
        if let Some(status) = output.status.code() {
            eprintln!("with status {}", status);
        } else {
            eprintln!("without status");
        }

        match String::from_utf8(output.stderr) {
            Ok(output) => eprintln!("Stderr: {}", output),
            Err(err) => eprintln!("Stderr is not UTF-8: {}", err),
        }

        return None;
    }

    let output = match String::from_utf8(output.stdout) {
        Ok(output) => output,
        Err(err) => {
            eprintln!("Filter script stdout is not UTF-8: {}", err);
            return None;
        }
    };

    let mut output = output.split('\n');
    Some((
        output.next()?.to_string(),
        output.next()?.to_string(),
        output.next()?.to_string(),
    ))
}
