# rescrobbled

Rescrobbled is a Last.fm scrobbler daemon written in Rust. Among other things, due to sharing a Spotify account (I know, I know), I needed a way to scrobble to [Last.fm](https://last.fm) without connecting the Spotify account to my Last.fm account. Rescrobbled detects active media players running on D-Bus using `MPRIS`, automatically updates "now playing" status, and scrobbles songs as they play.
