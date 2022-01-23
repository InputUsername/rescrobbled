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

use std::collections::HashSet;
use std::thread;
use std::time::Duration;

use mpris::{PlaybackStatus, Player, PlayerFinder};

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

/// Determine if the unique part of the D-Bus bus name, ie. the part
/// after `org.mpris.MediaPlayer2.`, is whitelisted.
///
/// This takes into account the possibility of multiple player instances:
/// it checks both the name, and the name with the instance part
/// (something like `.instance123`) stripped off.
fn is_bus_name_whitelisted<'p>(player: &'p Player, whitelist: &HashSet<String>) -> bool {
    let bus_name = player
        .bus_name()
        .as_cstr()
        .to_str()
        .unwrap_or("")
        .trim_start_matches(BUS_NAME_PREFIX);

    let without_instance = bus_name
        .rsplit_once(".")
        .map(|(name, _instance)| name)
        .unwrap_or(bus_name);

    whitelist.contains(bus_name) || whitelist.contains(without_instance)
}

/// Determine if a player's MPRIS identity or the unique part
/// of its D-Bus bus name are whitelisted.
fn is_whitelisted(config: &Config, player: &Player) -> bool {
    if let Some(ref whitelist) = config.player_whitelist {
        if !whitelist.is_empty() {
            return whitelist.contains(player.identity())
                || is_bus_name_whitelisted(player, whitelist);
        }
    }
    true
}

/// Wait for any (whitelisted) player to become active again.
pub fn wait_for_player<'f>(config: &Config, finder: &'f PlayerFinder) -> Player<'f> {
    loop {
        let players = match finder.find_all() {
            Ok(players) => players,
            _ => {
                thread::sleep(INIT_WAIT_TIME);
                continue;
            }
        };

        for player in players {
            if is_active(&player) && is_whitelisted(config, &player) {
                return player;
            }
        }

        thread::sleep(INIT_WAIT_TIME);
    }
}
