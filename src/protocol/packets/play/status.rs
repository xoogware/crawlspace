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

use uuid::Uuid;

use crate::protocol::{
    datatypes::{Bounded, VarInt},
    Encode, Packet, Property,
};

use super::Gamemode;

#[derive(Debug)]
pub struct PlayerInfoUpdateC<'a> {
    pub players: &'a [PlayerStatus<'a>],
}

#[derive(Debug)]
pub struct PlayerStatus<'a> {
    uuid: Uuid,
    actions: Vec<PlayerAction<'a>>,
}

#[derive(Debug)]
enum PlayerAction<'a> {
    AddPlayer {
        name: Bounded<&'a str, 16>,
        properties: &'a [Property<'a>],
    },
    UpdateGamemode {
        game_mode: VarInt,
    },
    UpdateListed {
        listed: bool,
    },
    UpdateLatency {
        latency: VarInt,
    },
}

impl Packet for PlayerInfoUpdateC<'_> {
    const ID: i32 = 0x3E;
}

// "I'm a Never-Nester"
// The unwavering Minecraft Protocol:
impl Encode for PlayerInfoUpdateC<'_> {
    fn encode(&self, mut w: impl std::io::Write) -> color_eyre::eyre::Result<()> {
        let actions = self.players.iter().fold(0, |acc, p| {
            acc | p.actions.iter().fold(0, |acc2, a| acc2 | a.mask())
        });

        actions.encode(&mut w)?;
        VarInt(self.players.len() as i32).encode(&mut w)?;

        for PlayerStatus { uuid, actions } in self.players {
            uuid.encode(&mut w)?;

            for action in actions {
                match action {
                    PlayerAction::AddPlayer { name, properties } => {
                        name.encode(&mut w)?;
                        VarInt(properties.len() as i32).encode(&mut w)?;
                        for p in *properties {
                            p.encode(&mut w)?;
                        }
                    }
                    PlayerAction::UpdateGamemode { game_mode } => {
                        game_mode.encode(&mut w)?;
                    }
                    PlayerAction::UpdateListed { listed } => {
                        listed.encode(&mut w)?;
                    }
                    PlayerAction::UpdateLatency { latency } => {
                        latency.encode(&mut w)?;
                    }
                }
            }
        }

        Ok(())
    }
}

impl PlayerAction<'_> {
    const fn mask(&self) -> i8 {
        match self {
            PlayerAction::AddPlayer { .. } => 0x01,
            PlayerAction::UpdateGamemode { .. } => 0x04,
            PlayerAction::UpdateListed { .. } => 0x08,
            PlayerAction::UpdateLatency { .. } => 0x10,
        }
    }
}

impl<'a> PlayerStatus<'a> {
    pub fn for_player(player: Uuid) -> Self {
        Self {
            uuid: player,
            actions: Vec::new(),
        }
    }

    pub fn add_player(mut self, name: &'a str, props: &'a [Property]) -> Self {
        self.actions.push(PlayerAction::AddPlayer {
            name: Bounded::<&'a str, 16>(name),
            properties: props,
        });
        self
    }

    pub fn update_gamemode(mut self, gamemode: Gamemode) -> Self {
        self.actions.push(PlayerAction::UpdateGamemode {
            game_mode: VarInt(u8::from(gamemode) as i32),
        });
        self
    }

    pub fn update_listed(mut self, listed: bool) -> Self {
        self.actions.push(PlayerAction::UpdateListed { listed });
        self
    }

    pub fn update_latency(mut self, latency: i32) -> Self {
        self.actions.push(PlayerAction::UpdateLatency {
            latency: VarInt(latency),
        });
        self
    }
}
