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

use std::io::Write;
use std::process::{Command, Stdio};

use anyhow::{Context, Result, anyhow, bail};

use crate::config::Config;
use crate::track::Track;

#[derive(Debug, PartialEq)]
pub enum FilterResult {
    Filtered(Track),
    NotFiltered(Track),
    Ignored,
}

pub fn filter_metadata(config: &Config, track: Track) -> Result<FilterResult> {
    if config.filter_script.is_none() {
        return Ok(FilterResult::NotFiltered(track));
    }

    let path = config.filter_script.as_ref().unwrap();

    let mut child = Command::new(config.filter_script.as_ref().unwrap())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .with_context(|| format!("Failed to run filter script at {}", path.display()))?;

    let mut stdin = child
        .stdin
        .take()
        .ok_or_else(|| anyhow!("Failed to get an stdin handle for the filter script"))?;

    let buffer = format!("{}\n{}\n{}\n", track.artist(), track.title(), track.album());
    stdin
        .write_all(buffer.as_bytes())
        .context("Failed to write track metadata to filter script stdin")?;

    // Close child's stdin to prevent endless waiting
    drop(stdin);

    let output = child
        .wait_with_output()
        .context("Failed to retrieve output from filter script")?;

    if !output.status.success() {
        let mut message = "Filter script returned unsuccessully ".to_owned();
        if let Some(status) = output.status.code() {
            message += &format!("with status: {}\n", status);
        } else {
            message += "without status\n";
        }

        match String::from_utf8(output.stderr) {
            Ok(output) => message += &format!("Stderr: {}", output),
            Err(err) => message += &format!("Stderr is not valid UTF-8: {}", err),
        }

        bail!(message);
    }

    let output = String::from_utf8(output.stdout)
        .context("Filter script stdout is not valid UTF-8")?;

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

        config.filter_script = Some(path);

        assert_eq!(
            filter_metadata(&config, Track::new("lorem", "ipsum", "dolor")).unwrap(),
            FilterResult::Filtered(Track::new(
                "Artist=lorem",
                "Title=ipsum",
                "Album=dolor"
            ))
        );

        // Script that produces no output should result in `FilterResult::Ignored`

        let path_ignore = temp_dir.path().join("filter_ignore.sh");
        const FILTER_SCRIPT_IGNORE: &str = "#!/usr/bin/bash
true
";

        fs::write(&path_ignore, FILTER_SCRIPT_IGNORE).unwrap();
        fs::set_permissions(&path_ignore, fs::Permissions::from_mode(0o755)).unwrap();

        config.filter_script = Some(path_ignore);

        assert_eq!(
            filter_metadata(&config, Track::new("lorem", "ipsum", "dolor")).unwrap(),
            FilterResult::Ignored
        );

        // Not using a filter script should result in `FilterResult::NotFiltered`

        config.filter_script = None;

        assert_eq!(
            filter_metadata(&config, Track::new("lorem", "ipsum", "dolor")).unwrap(),
            FilterResult::NotFiltered(Track::new(
                "lorem", "ipsum", "dolor"
            ))
        );
    }
}
