use std::io::Write;
use std::process::{Command, Stdio};

use crate::config::Config;
use crate::track::Track;

#[derive(Debug, PartialEq)]
pub enum FilterResult {
    Filtered(Track),
    NotFiltered(Track),
    Ignored,
}

pub fn filter_metadata(config: &Config, track: Track) -> Result<FilterResult, String> {
    if config.filter_script.is_none() {
        return Ok(FilterResult::NotFiltered(track));
    }

    let mut child = Command::new(config.filter_script.as_ref().unwrap())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .map_err(|err| format!("Failed to run filter script: {}", err))?;

    let mut stdin = child.stdin
        .take()
        .ok_or_else(|| "Failed to get a stdin handle for the filter script".to_owned())?;

    let buffer = format!("{}\n{}\n{}\n", track.artist(), track.title(), track.album());
    stdin
        .write_all(buffer.as_bytes())
        .map_err(|err| format!("Failed to write metadata to filter script stdin: {}", err))?;

    // Close child's stdin to prevent endless waiting
    drop(stdin);

    let output = child
        .wait_with_output()
        .map_err(|err| format!("Failed to retrieve output from filter script: {}", err))?;

    if !output.status.success() {
        let mut message = "Filter script returned unsuccessfully ".to_owned();
        if let Some(status) = output.status.code() {
            message += &format!("with status {}", status);
        } else {
            message += "without status";
        }

        match String::from_utf8(output.stderr) {
            Ok(output) => message += &format!("Stderr: {}", output),
            Err(err) => message += &format!("Stderr is not UTF-8: {}", err),
        }

        return Err(message);
    }

    let output = String::from_utf8(output.stdout)
        .map_err(|err| format!("Filter script stdout is not UTF-8: {}", err))?;

    let mut output = output.split('\n');
    match (output.next(), output.next(), output.next()) {
        (Some(artist), Some(title), Some(album)) => {
            Ok(FilterResult::Filtered(Track::new(artist, title, album)))
        }
        _ => Ok(FilterResult::Ignored)
    }
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

        assert_eq!(
            filter_metadata(&config, Track::new("lorem", "ipsum", "dolor")),
            Ok(FilterResult::Filtered(Track::new("Artist=lorem", "Title=ipsum", "Album=dolor")))
        );

        // Script that produces no output should result in `FilterResult::Ignored`

        let path_ignore = temp_dir.path().join("filter_ignore.sh");
        const FILTER_SCRIPT_IGNORE: &str = "#!/usr/bin/bash
true
";

        fs::write(&path_ignore, FILTER_SCRIPT_IGNORE).unwrap();
        fs::set_permissions(&path_ignore, fs::Permissions::from_mode(0o755)).unwrap();

        config.filter_script = Some(path_ignore.to_string_lossy().into_owned());

        assert_eq!(
            filter_metadata(&config, Track::new("lorem", "ipsum", "dolor")),
            Ok(FilterResult::Ignored)
        );

        // Not using a filter script should result in `FilterResult::NotFiltered`

        config.filter_script = None;

        assert_eq!(
            filter_metadata(&config, Track::new("lorem", "ipsum", "dolor")),
            Ok(FilterResult::NotFiltered(Track::new("lorem", "ipsum", "dolor")))
        );
    }
}
