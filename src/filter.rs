use std::io::Write;
use std::process::{Command, Stdio};

use crate::config::Config;

pub fn filter_metadata(
    config: &Config,
    artist: &str,
    title: &str,
    album: &str,
) -> Option<(String, String, String)> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_script() {
        use std::fs;
        use std::os::unix::fs::PermissionsExt;

        const FILTER_SCRIPT: &str = "#!/usr/bin/bash
read artist
read title
read album
echo \"Artist=$artist\"
echo \"Title=$title\"
echo \"Album=$album\"
";

        let temp_dir = tempfile::tempdir().unwrap();
        let path = temp_dir.path().join("filter.sh");

        fs::write(&path, FILTER_SCRIPT).unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();

        let mut config = Config::default();
        config.filter_script = Some(path.to_string_lossy().into_owned());

        let (artist, title, album) = filter_metadata(&config, "lorem", "ipsum", "dolor").unwrap();

        assert_eq!(artist, "Artist=lorem");
        assert_eq!(title, "Title=ipsum");
        assert_eq!(album, "Album=dolor");
    }
}
