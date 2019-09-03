# rescrobbled

Rescrobbled is a Last.fm scrobbler daemon written in Rust. Among other things, due to sharing a Spotify account (I know, I know), I needed a way to scrobble to [Last.fm](https://last.fm) without connecting the Spotify account to my Last.fm account. Rescrobbled detects active media players running on D-Bus using `MPRIS`, automatically updates "now playing" status, and scrobbles songs as they play.

# Configuration

Rescrobbled expects a configuration file at `~/.config/rescrobbled/config.toml` with the following format:
```toml
api-key = "Last.fm API key"
api-secret = "Last.fm API secret"
lb-token = "ListenBrainz API token" # optional
enable-notifications = false # optional
min-play-time = { secs: 0, nanos: 0 } # optional
```

By default, track submission respects Last.fm's recommended behavior; songs should only be scrobbled if they have been playing for at least half their duration, or for 4 minutes, whichever comes first. Using `min-play-time` you can override this.

# Systemd unit file

You can put the provided [systemd unit file](https://github.com/InputUsername/rescrobbled/blob/master/rescrobbled.service) in the `~/.config/systemd/user/` directory and start it with `systemctl --user start rescrobbled.service`.