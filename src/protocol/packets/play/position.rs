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

use crate::protocol::{Decode, Packet, PacketDirection, PacketState};

#[derive(Debug)]
pub struct SetPlayerPositionS {
    pub x: f64,
    pub feet_y: f64,
    pub z: f64,
    pub on_ground: bool,
}

impl Packet for SetPlayerPositionS {
    const ID: &'static str = "minecraft:move_player_pos";
    const STATE: PacketState = PacketState::Play;
    const DIRECTION: PacketDirection = PacketDirection::Serverbound;
}

impl Decode<'_> for SetPlayerPositionS {
    fn decode(r: &mut &'_ [u8]) -> color_eyre::eyre::Result<Self>
    where
        Self: Sized,
    {
        Ok(Self {
            x: r.read_f64::<BigEndian>()?,
            feet_y: r.read_f64::<BigEndian>()?,
            z: r.read_f64::<BigEndian>()?,
            on_ground: r.read_u8()? == 1,
        })
    }
}

#[derive(Debug)]
pub struct SetPlayerPositionAndRotationS {
    pub x: f64,
    pub feet_y: f64,
    pub z: f64,
    pub yaw: f32,
    pub pitch: f32,
    pub on_ground: bool,
}

impl Packet for SetPlayerPositionAndRotationS {
    const ID: &'static str = "minecraft:move_player_pos_rot";
    const STATE: PacketState = PacketState::Play;
    const DIRECTION: PacketDirection = PacketDirection::Serverbound;
}

impl Decode<'_> for SetPlayerPositionAndRotationS {
    fn decode(r: &mut &'_ [u8]) -> color_eyre::eyre::Result<Self>
    where
        Self: Sized,
    {
        Ok(Self {
            x: r.read_f64::<BigEndian>()?,
            feet_y: r.read_f64::<BigEndian>()?,
            z: r.read_f64::<BigEndian>()?,
            yaw: r.read_f32::<BigEndian>()?,
            pitch: r.read_f32::<BigEndian>()?,
            on_ground: r.read_u8()? == 1,
        })
    }
}
