// Copyright (C) 2026 Koen Bolhuis
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

use std::thread;
use std::time::Duration;

use mpris::{PlaybackStatus, Player, PlayerFinder};
use regex::RegexSet;

use crate::config::Config;

const INIT_WAIT_TIME: Duration = Duration::from_secs(1);

const BUS_NAME_PREFIX: &str = "org.mpris.MediaPlayer2.";

/// Determine if a player is running and actually playing music.
pub fn is_active(player: &Player) -> bool {
    if !player.is_running() {
        return false;
    }

    matches!(player.get_playback_status(), Ok(PlaybackStatus::Playing))
}

/// Determine if the MPRIS identity or the unique part of the D-Bus bus name
/// (i.e. the part after `org.mpris.MediaPlayer2.`) is contained in the regex set.
///
/// This takes into account the possibility of multiple player instances:
/// it checks both the name, and the name with the instance part
/// (something like `.instance123`) stripped off.
fn regex_set_contains(set: &RegexSet, player: &Player) -> bool {
    let bus_name = player.bus_name().trim_start_matches(BUS_NAME_PREFIX);

    let without_instance = bus_name
        .rsplit_once('.')
        .map(|(name, _instance)| name)
        .unwrap_or(bus_name);

    set.is_match(player.identity()) || set.is_match(bus_name) || set.is_match(without_instance)
}

/// Determine if a player's MPRIS identity or its D-Bus bus name are whitelisted.
fn is_whitelisted(config: &Config, player: &Player) -> bool {
    if let Some(ref whitelist) = config.player_whitelist
        && !whitelist.is_empty()
    {
        return regex_set_contains(whitelist, player);
    }
    true
}

// Determine if a player's MPRIS identity or its D-Bus bus name are ignorelisted.
fn is_ignorelisted(config: &Config, player: &Player) -> bool {
    if let Some(ref ignorelist) = config.player_ignorelist
        && !ignorelist.is_empty()
    {
        return regex_set_contains(ignorelist, player);
    }
    false
}

/// Wait for any (whitelisted, not ignorelisted) player to become active again.
pub fn wait_for_player(config: &Config, finder: &PlayerFinder) -> Player {
    loop {
        let players = match finder.iter_players() {
            Ok(players) => players,
            _ => {
                thread::sleep(INIT_WAIT_TIME);
                continue;
            }
        };

        for player in players {
            if let Ok(player) = player
                && is_active(&player)
                && is_whitelisted(config, &player)
                && !is_ignorelisted(config, &player)
            {
                return player;
            }
        }

        thread::sleep(INIT_WAIT_TIME);
    }
}
