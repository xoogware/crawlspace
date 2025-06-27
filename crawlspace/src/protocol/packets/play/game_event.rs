/*
 * Copyright (c) 2024 Andrew Brower.
 * This file is part of Crawlspace.
 *
 * Crawlspace is free software: you can redistribute it and/or
 * modify it under the terms of the GNU Affero General Public
 * License as published by the Free Software Foundation, either
 * version 3 of the License, or (at your option) any later version.
 *
 * Crawlspace is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU
 * Affero General Public License for more details.
 *
 * You should have received a copy of the GNU Affero General Public
 * License along with Crawlspace. If not, see
 * <https://www.gnu.org/licenses/>.
 */

use crawlspace_macro::Packet;

use crate::protocol::{Encode, Packet, PacketDirection, PacketState};

use super::Gamemode;

#[derive(Debug, Packet)]
#[packet(id = "minecraft:game_event", clientbound, state = "PacketState::Play")]
pub struct GameEventC {
    event: u8,
    value: f32,
}

impl Encode for GameEventC {
    fn encode(&self, mut w: impl std::io::Write) -> color_eyre::eyre::Result<()> {
        self.event.encode(&mut w)?;
        self.value.encode(&mut w)?;
        Ok(())
    }
}

#[derive(Debug)]
#[repr(u8)]
#[expect(unused)]
pub enum GameEvent {
    NoRespawnBlockAvailable,
    BeginRaining,
    EndRaining,
    ChangeGamemode(Gamemode),
    /// true to roll credits
    WinGame(bool),
    Demo(DemoCommand),
    ArrowHit,
    /// 0 to 1
    RainLevelChange(f32),
    /// 0 to 1
    ThunderLevelChange(f32),
    PlayPufferfishStingSound,
    PlayElderGuardianAppearance,
    /// true to immediately respawn, false to enable respawn screen
    EnableRespawn(bool),
    /// true for limited crafting false for normal
    LimitedCrafting(bool),
    /// the big one!
    StartWaitingForLevelChunks,
}

#[derive(Debug)]
#[expect(unused)]
pub enum DemoCommand {
    ShowWelcome = 0,
    TellMovement = 101,
    TelJump = 102,
    TellInv = 103,
    TellOver = 104,
}

impl From<GameEvent> for GameEventC {
    fn from(value: GameEvent) -> Self {
        let event_id = unsafe { *(&value as *const GameEvent as *const u8) };

        let value = match value {
            GameEvent::ChangeGamemode(g) => u8::from(g) as f32,
            GameEvent::WinGame(c) | GameEvent::EnableRespawn(c) | GameEvent::LimitedCrafting(c) => {
                if c {
                    1.0
                } else {
                    0.0
                }
            }
            GameEvent::Demo(cmd) => cmd as u8 as f32,
            GameEvent::RainLevelChange(l) | GameEvent::ThunderLevelChange(l) => l,
            _ => 0.0,
        };

        GameEventC {
            event: event_id,
            value,
        }
    }
}
