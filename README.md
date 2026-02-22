# rescrobbled

[![License](https://img.shields.io/github/license/InputUsername/rescrobbled)](https://github.com/InputUsername/rescrobbled/blob/master/LICENSE)
[![Crates.io](https://img.shields.io/crates/v/rescrobbled)](https://crates.io/crates/rescrobbled)
[![CI](https://github.com/InputUsername/rescrobbled/actions/workflows/ci.yml/badge.svg)](https://github.com/InputUsername/rescrobbled/actions/workflows/ci.yml)

Rescrobbled is a music scrobbler daemon. It detects active media players running on D-Bus using [MPRIS](https://specifications.freedesktop.org/mpris-spec/latest/), automatically updates "now playing" status, and scrobbles songs to [Last.fm](https://last.fm) or [ListenBrainz](https://listenbrainz.org)-compatible services as they play.

Among other things, due to sharing a Spotify account (I know, I know), I needed a way to scrobble to Last.fm without connecting the Spotify account to my Last.fm account. Rescrobbled offers a simple solution for this.

## Installation

You can download one of the prebuilt binaries [here](https://github.com/InputUsername/rescrobbled/releases). The binary can be placed anywhere you like.

Rescrobbled is available on [crates.io](https://crates.io/crates/rescrobbled):
```
cargo install rescrobbled
```

Alternatively you can install from source using `cargo install --path .` from the crate root.

There is also an [AUR package](https://aur.archlinux.org/packages/rescrobbled-git/) by [brycied00d](https://github.com/brycied00d), which should always build the latest version of rescrobbled from this repository.

## Configuration

Rescrobbled expects a configuration file at `~/.config/rescrobbled/config.toml` with the following format:
```toml
lastfm-key = "Last.fm API key"
lastfm-secret = "Last.fm API secret"
min-play-time = 0
player-whitelist = [ "Player MPRIS identity or bus name" ]
filter-script = "path/to/script"
use-track-start-timestamp = false

[[listenbrainz]]
url = "Custom API URL"
token = "User token"
```

All settings are optional, although rescrobbled isn't very useful without Last.fm or ListenBrainz credentials. ;-)

If the config file doesn't exist, rescrobbled will generate an example config for you when you run it for the first time.

<table>
<thead>
    <tr>
        <th>Option</th>
        <th>Description</th>
    </tr>
</thead>
<tbody>
    <tr>
        <td>
            <p><code>lastfm-key</code>, <code>lastfm-secret</code></p>
        </td>
        <td>To use rescrobbled with Last.fm, you'll need a Last.fm API key and secret. These can be obtained <a href="https://www.last.fm/api/account/create">here</a>.</td>
    </tr>
    <tr>
        <td><code>min-play-time</code></td>
        <td>
            <p>Minimum play time in seconds before a song is scrobbled.</p>
            <p>By default, track submission respects Last.fm's recommended behavior: songs should only be scrobbled if they have been playing for at least half their duration, or for 4 minutes, whichever comes first. Using <code>min-play-time</code> you can override this.</p>
        </td>
    </tr>
    <tr>
        <td><code>player-whitelist</code></td>
        <td>
            <p>If empty or ommitted, music from all players will be scrobbled; otherwise, rescrobbled will only listen to players in this list.</p>
            <p>A CLI application like <a href="https://github.com/altdesktop/playerctl">playerctl</a> can be used to determine a player's name for the whitelist. To do so, start playing a song and run the following command:</p>
            <pre><code>playerctl --list-all</code></pre>
        </td>
    </tr>
    <tr>
        <td><code>filter-script</code></td>
        <td>
            <p>The <code>filter-script</code> will be run before updating status and before submitting tracks.
            It receives the following properties on consecutive lines of its standard input (separated by <code>\n</code>):
            <ul>
                <li>artist;</li>
                <li>song title;</li>
                <li>album name;</li>
                <li>zero or more comma-separated (<code>,</code>) genre(s)</li>
            </ul>
            </p>
            <p>The script should write the filtered artist, song title and album name on corresponding lines of its standard output.
            This can be used to clean up song names, for example removing "remastered" and similar suffixes.
            If the filter script does not return any output, the current track will be ignored.</p>
            <p>A number of example scripts can be found in the <a href="https://github.com/InputUsername/rescrobbled/tree/master/filter-script-examples"><code>filter-script-examples</code></a> directory.</p>
        </td>
    </tr>
    <tr>
        <td><code>use-track-start-timestamp</code></td>
        <td>By default, tracks are submitted with a timestamp of the submission time. By setting <code>use-track-start-timestamp</code> to <code>true</code>, tracks are instead submitted with the time the track originally started playing. This is currently Last.fm-only.</td>
    </tr>
    <tr>
        <td><code>[[listenbrainz]]</code></td>
        <td>
            <p>You can specify one or more ListenBrainz instances by repeating this option. Each definition needs at least a <code>token</code>. You can set <code>url</code> to use a custom API URL (eg. for use with custom ListenBrainz instances or services like <a href="https://github.com/krateng/maloja">Maloja</a>). If the URL is not provided, it defaults to the ListenBrainz.org instance.</p>
            <p>If you only want to use ListenBrainz.org, you can set the <code>listenbrainz-token</code> option as a shorthand instead.</p>
            <p>For ListenBrainz.org, the user token can be found <a href="https://listenbrainz.org/profile/">here</a>. Other services might do this differently, refer to their documentation for more info.</p>
        </td>
    </tr>
</tbody>
</table>

> [!NOTE]
> Due to the way TOML works, the `[[listenbrainz]]` definitions need to be the last thing in your config file.

### Environment variables

Some options can be set using environment variables. The following options are supported:
| Option | Environment variable |
|---|---|
| `lastfm-key`, `lastfm-secret` | `LASTFM_KEY`, `LASTFM_SECRET` |
| `listenbrainz-token` | `LISTENBRAINZ_TOKEN` |
| `min-play-time` | `MIN_PLAY_TIME` |
| `filter-script` | `FILTER_SCRIPT` |
| `use-track-start-timestamp` | `USE_TRACK_START_TIMESTAMP` |

### Loading secrets from files

Secrets can alternatively be loaded from files. This is useful for secret managers like [agenix](https://github.com/ryantm/agenix).
Use the following options for this:
- `lastfm-key-file`
- `lastfm-secret-file`
- `listenbrainz-token-file`
- `[[listenbrainz]] token-file`

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

Issues and pull requests are more than welcome! Development happens on the [`development`](https://github.com/InputUsername/rescrobbled/tree/development) branch, so please create pull requests against that.
All contributions will be licensed under GPLv3.

## License

GPL-3.0, see [`LICENSE`](https://github.com/InputUsername/rescrobbled/blob/master/LICENSE).
