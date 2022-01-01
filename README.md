# rescrobbled

[![License](https://img.shields.io/github/license/InputUsername/rescrobbled)](https://github.com/InputUsername/rescrobbled/blob/master/LICENSE)
[![Crates.io](https://img.shields.io/crates/v/rescrobbled)](https://crates.io/crates/rescrobbled)
[![CI](https://github.com/InputUsername/rescrobbled/actions/workflows/ci.yml/badge.svg)](https://github.com/InputUsername/rescrobbled/actions/workflows/ci.yml)

Rescrobbled is a music scrobbler daemon. It detects active media players running on D-Bus using `MPRIS`, automatically updates "now playing" status, and scrobbles songs to [Last.fm](https://last.fm) or [ListenBrainz](https://listenbrainz.org) as they play.

Among other things, due to sharing a Spotify account (I know, I know), I needed a way to scrobble to Last.fm without connecting the Spotify account to my Last.fm account. Rescrobbled offers a simple solution for this.

## Installation

You can download one of the prebuilt binaries [here](https://github.com/InputUsername/rescrobbled/releases). The binary can be placed anywhere you like.

Rescrobbled is available on [Crates.io](https://crates.io/crates/rescrobbled): `cargo install rescrobbled`

Alternatively you can install from source using `cargo install --path .` from the crate root.

There is also an [AUR package](https://aur.archlinux.org/packages/rescrobbled-git/) by [brycied00d](https://github.com/brycied00d), which should always build the latest version of rescrobbled from this repository.

## Configuration

Rescrobbled expects a configuration file at `~/.config/rescrobbled/config.toml` with the following format:
```toml
lastfm-key = "Last.fm API key"
lastfm-secret = "Last.fm API secret"
enable-notifications = false
min-play-time = 0
player-whitelist = [ "Player MPRIS identity or bus name" ]
filter-script = "path/to/script"

[[listenbrainz]]
url = "Custom API URL"
token = "User token"
```

All settings are optional, although rescrobbled isn't very useful without Last.fm or ListenBrainz credentials. ;-)

If the config file doesn't exist, rescrobbled will generate an example config for you when you run it for the first time.

`lastfm-key`, `lastfm-secret`

To use rescrobbled with Last.fm, you'll need a Last.fm API key and secret. These can be obtained [here](https://www.last.fm/api/account/create).

`enable-notifications`

Set this to `true` to show desktop notifications when a song starts playing: useful if your music player does not support notifications. Defaults to `false`.

`min-play-time`

Minimum play time in seconds before a song is scrobbled.

By default, track submission respects Last.fm's recommended behavior: songs should only be scrobbled if they have been playing for at least half their duration, or for 4 minutes, whichever comes first. Using `min-play-time` you can override this.

`player-whitelist`

If empty or ommitted, music from all players will be scrobbled; otherwise, rescrobbled will only listen to players in this list.

A CLI application like `playerctl` can be used to determine a player's MPRIS identity for the whitelist. To do so start playing a song and run the following command:
```
playerctl --list-all
```

`filter-script`

The `filter-script` will be run before submitting tracks.
It receives the artist, song title and album name on consecutive lines of its standard input
(in that order). The script should provide the filtered metadata on corresponding lines of its standard output.
This can be used to clean up song names, for example removing "remastered" and similar suffixes.
If the filter script does not return any output, the current track will be ignored.

A number of example scripts can be found in the [`filter-script-examples`](https://github.com/InputUsername/rescrobbled/tree/master/filter-script-examples) directory.

`[[listenbrainz]]`

You can specify one or more ListenBrainz instances by repeating this option. Each definition needs at least a `token`. You can set `url` to use a custom API URL (eg. for use with custom ListenBrainz instances or services like [Maloja](https://github.com/krateng/maloja)). If the URL is not provided, it defaults to the ListenBrainz.org instance.

If you only want to use ListenBrainz.org, you can set the `listenbrainz-token` option as a shorthand instead.

For ListenBrainz.org, the user token can be found [here](https://listenbrainz.org/profile/). Other services might do this differently, refer to their documentation for more info.

*Note: due to the way TOML works, these need to be the last thing in your config file.*

## Usage

To make sure that rescrobbled can scrobble to Last.fm, you need to run the program in a terminal. This will prompt you for your Last.fm username and password, and authenticate with Last.fm. A long-lasting session token is then obtained, which will be used on subsequent runs instead of your username/password. The session token is stored in `~/.config/rescrobbled/session`.

If you want to run rescrobbled as a daemon, you can put the provided [systemd unit file](https://github.com/InputUsername/rescrobbled/blob/master/rescrobbled.service) in the `~/.config/systemd/user/` directory.
Change `ExecStart` to point to the location of the binary, as necessary. Then, to enable the program to run at startup, use:
```
systemctl --user enable rescrobbled.service
```
You can run it in the current session using:
```
systemctl --user start rescrobbled.service
```

## Project resources

- [Issues](https://github.com/InputUsername/rescrobbled/issues)
- [Changelog](https://github.com/InputUsername/rescrobbled/blob/master/CHANGELOG.md)
- [Releases](https://github.com/InputUsername/rescrobbled/releases)

Issues and pull requests are more than welcome! Development happens on the [`development`](https://github.com/InputUsername/rescrobbled/tree/development) branch, so please create PRs against that.
All contributions will be licensed under GPLv3.

## License

GPL-3.0, see [`LICENSE`](https://github.com/InputUsername/rescrobbled/blob/master/LICENSE).
