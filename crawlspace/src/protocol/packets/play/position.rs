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

use byteorder::{BigEndian, ReadBytesExt};
use crawlspace_macro::{Decode, Packet};

use crate::protocol::{Decode, Packet, PacketDirection, PacketState};

#[derive(Debug, Packet, Decode)]
#[packet(
    id = "minecraft:move_player_pos",
    serverbound,
    state = "PacketState::Play"
)]
pub struct SetPlayerPositionS {
    pub x: f64,
    pub feet_y: f64,
    pub z: f64,
    pub on_ground: PosRotFlags,
}

#[derive(Debug, Packet, Decode)]
#[packet(
    id = "minecraft:move_player_pos_rot",
    serverbound,
    state = "PacketState::Play"
)]
pub struct SetPlayerPositionAndRotationS {
    pub x: f64,
    pub feet_y: f64,
    pub z: f64,
    pub yaw: f32,
    pub pitch: f32,
    pub flags: PosRotFlags,
}

#[derive(Clone, Debug)]
pub struct PosRotFlags {
    on_ground: bool,
    against_wall: bool,
}

impl Decode<'_> for PosRotFlags {
    fn decode(r: &mut &'_ [u8]) -> color_eyre::eyre::Result<Self>
    where
        Self: Sized,
    {
        let field = u8::decode(r)?;

        Ok(PosRotFlags {
            on_ground: field & 0b00000001 != 0,
            against_wall: field & 0b00000010 != 0,
        })
    }
}
