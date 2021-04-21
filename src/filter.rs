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

        let mut config = Config::default();
        let temp_dir = tempfile::tempdir().unwrap();

        let path = temp_dir.path().join("filter.sh");
        const FILTER_SCRIPT: &str = "#!/usr/bin/bash
read artist
read title
read album
echo \"Artist=$artist\"
echo \"Title=$title\"
echo \"Album=$album\"
";

        fs::write(&path, FILTER_SCRIPT).unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();

        config.filter_script = Some(path.to_string_lossy().into_owned());

        let (artist, title, album) = filter_metadata(&config, "lorem", "ipsum", "dolor").unwrap();

        assert_eq!(artist, "Artist=lorem");
        assert_eq!(title, "Title=ipsum");
        assert_eq!(album, "Album=dolor");

        // Script that produces no output should result in
        // `filter_metadata` returning `None`

        let path_ignore = temp_dir.path().join("filter_ignore.sh");
        const FILTER_SCRIPT_IGNORE: &str = "#!/usr/bin/bash
true
";

        fs::write(&path_ignore, FILTER_SCRIPT_IGNORE).unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();

        config.filter_script = Some(path_ignore.to_string_lossy().into_owned());

        assert!(filter_metadata(&config, "lorem", "ipsum", "dolor").is_none());
    }
}
