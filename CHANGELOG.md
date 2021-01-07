# Changelog

## v0.3.0 (unreleased)

- Fixed a bug where a single song on repeat only scrobbled once
- Rescrobbled now creates the config file if it doesn't exist

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
