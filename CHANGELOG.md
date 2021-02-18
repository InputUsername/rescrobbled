# Changelog

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
