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

use crate::protocol::{
    datatypes::{Bounded, Position, VarInt},
    Encode, Packet,
};

#[derive(Debug)]
pub struct LoginPlayC<'a> {
    /// The player's Entity ID (EID).
    pub entity_id: i32,
    pub is_hardcore: bool,
    /// Identifiers for all dimensions on the server.
    pub dimension_names: Vec<Bounded<&'a str>>,
    /// Was once used by the client to draw the player list, but now is ignored.
    pub max_players: VarInt,
    /// Render distance (2-32).
    pub view_distance: VarInt,
    /// The distance that the client will process specific things, such as entities.
    pub simulation_distance: VarInt,
    /// If true, a Notchian client shows reduced information on the debug screen. For servers in development, this should almost always be false.
    pub reduced_debug_info: bool,
    /// Set to false when the doImmediateRespawn gamerule is true.
    pub enable_respawn_screen: bool,
    /// Whether players can only craft recipes they have already unlocked. Currently unused by the client.
    pub do_limited_crafting: bool,
    /// okay i don't feel like copying these anymore https://wiki.vg/Protocol#Login_.28play.29
    pub dimension_type: VarInt,
    pub dimension_name: Bounded<&'a str>,
    /// first 8 bytes of seed sha256 - probably unneeded?
    pub hashed_seed: i64,
    pub gamemode: Gamemode,
    pub previous_gamemode: Option<Gamemode>,
    pub is_debug: bool,
    pub is_superflat: bool,
    pub death_location: Option<DeathLocation<'a>>,
    pub portal_cooldown: VarInt,
    pub sea_level: VarInt,
    pub enforces_secure_chat: bool,
}

#[derive(Clone, Copy, Debug)]
pub enum Gamemode {
    Survival,
    Creative,
    Adventure,
    Spectator,
}

impl From<Gamemode> for u8 {
    fn from(value: Gamemode) -> Self {
        match value {
            Gamemode::Survival => 0,
            Gamemode::Creative => 1,
            Gamemode::Adventure => 2,
            Gamemode::Spectator => 3,
        }
    }
}

impl From<Gamemode> for i8 {
    fn from(value: Gamemode) -> Self {
        match value {
            Gamemode::Survival => 0,
            Gamemode::Creative => 1,
            Gamemode::Adventure => 2,
            Gamemode::Spectator => 3,
        }
    }
}

#[derive(Debug)]
pub struct DeathLocation<'a> {
    dimension_name: Bounded<&'a str>,
    death_location: Position,
}

impl Packet for LoginPlayC<'_> {
    const ID: i32 = 0x2C;
}

impl<'a> Encode for LoginPlayC<'a> {
    fn encode(&self, mut w: impl std::io::Write) -> color_eyre::eyre::Result<()> {
        self.entity_id.encode(&mut w)?;
        self.is_hardcore.encode(&mut w)?;
        VarInt(self.dimension_names.len() as i32).encode(&mut w)?;
        self.dimension_names.encode(&mut w)?;
        self.max_players.encode(&mut w)?;
        self.view_distance.encode(&mut w)?;
        self.simulation_distance.encode(&mut w)?;
        self.reduced_debug_info.encode(&mut w)?;
        self.enable_respawn_screen.encode(&mut w)?;
        self.do_limited_crafting.encode(&mut w)?;
        self.dimension_type.encode(&mut w)?;
        self.dimension_name.encode(&mut w)?;
        self.hashed_seed.encode(&mut w)?;
        u8::from(self.gamemode).encode(&mut w)?;

        match self.previous_gamemode {
            None => (-1).encode(&mut w)?,
            Some(g) => i8::from(g).encode(&mut w)?,
        }

        self.is_debug.encode(&mut w)?;
        self.is_superflat.encode(&mut w)?;

        match &self.death_location {
            Some(l) => {
                true.encode(&mut w)?;
                l.dimension_name.encode(&mut w)?;
                l.death_location.encode(&mut w)?;
            }
            None => {
                false.encode(&mut w)?;
            }
        }

        self.portal_cooldown.encode(&mut w)?;
        self.sea_level.encode(&mut w)?;
        self.enforces_secure_chat.encode(&mut w)?;

        Ok(())
    }
}
