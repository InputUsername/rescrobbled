# Changelog

## v0.5.0 (unreleased)

- Added support for multiple ListenBrainz instances
  - You can now specify multiple ListenBrainz instances, supporting custom installs
    and other scrobbling services that use a ListenBrainz compatible API
- Added a number of example filter scripts
- The auto-generated config file and session token file are now created with
  more restrictive permissions (`0600`)
- Internal refactoring
  - Improved code quality
  - Slightly improved error handling
- Cleaned up the README
  - Also documented where the session token is stored

## 0.4.0 (2021-05-07)

- Added ignore functionality for filter scripts:
  - Filter scripts that return nothing will cause the current track to be ignored/not scrobbled
  - This can be used to, for example, filter certain artists or songs entirely

## v0.3.3 (2021-05-06)

- Added `-v` (`--version`) command-line switch to get the program's version
- Released on crates.io

## v0.3.2 (2021-04-19)

- Fixed config template typos (`min_play_time` => `min-play-time`, `player_whitelist` => `player-whitelist`)

## v0.3.1 (2021-03-29)

- Fixed a typo in the config file template (`lastfm-token` => `lastfm-key`)

## v0.3.0 (2021-02-18)

- Fixed a bug where a single song on repeat only scrobbled once
- Rescrobbled now creates the config file if it doesn't exist
- Added the `filter-script` config option:
    - Rescrobbled will run this script to filter metadata before
      submitting it to Last.fm and/or ListenBrainz
    - The script receives artist, song title and album name on
      consecutive lines of its standard input (in that order)
    - It should produce filtered metadata on the corresponding
      lines of its standard output
    - Format might change in future updates, eg. to provide
      additional metadata

## v0.2.0 (2020-08-12)

- Improved usage instructions
- Renamed config options (old names still supported)
    - `api-key` => `lastfm-key`
    - `api-secret` => `lastfm-secret`
    - `lb-token` => `listenbrainz-token`
- Added music player whitelisting (by MPRIS identity or D-Bus bus name)
- Made Last.fm scrobbling optional

## v0.1.0 (2019-09-15)

Initial release
