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
use crawlspace_macro::Packet;

use crate::protocol::{
    datatypes::{Position, VarInt},
    Decode, Packet, PacketDirection, PacketState,
};

#[derive(Debug, Packet)]
#[packet(id = "minecraft:use_item_on", serverbound, state = "PacketState::Play")]
pub struct UseItemOnS {
    pub hand: Hand,
    pub location: Position,
    pub face: Face,
    pub cursor_x: f32,
    pub cursor_y: f32,
    pub cursor_z: f32,
    pub inside_block: bool,
    pub world_border_hit: bool,
    pub sequence: VarInt,
}

#[derive(Debug)]
pub enum Hand {
    Main,
    Off,
}

#[derive(thiserror::Error, Debug)]
pub enum HandParseError {
    #[error("Got unexpected hand index {0}")]
    Unexpected(i32),
}

impl TryFrom<VarInt> for Hand {
    type Error = HandParseError;

    fn try_from(value: VarInt) -> Result<Self, Self::Error> {
        match value.0 {
            0 => Ok(Hand::Main),
            1 => Ok(Hand::Off),
            i => Err(HandParseError::Unexpected(i)),
        }
    }
}

#[derive(Debug)]
pub enum Face {
    Bottom,
    Top,
    North,
    South,
    East,
    West,
}

#[derive(thiserror::Error, Debug)]
pub enum FaceParseError {
    #[error("Got unexpected face index {0}")]
    Unexpected(i32),
}

impl TryFrom<VarInt> for Face {
    type Error = FaceParseError;

    fn try_from(value: VarInt) -> Result<Self, Self::Error> {
        match value.0 {
            0 => Ok(Face::Bottom),
            1 => Ok(Face::Top),
            2 => Ok(Face::North),
            3 => Ok(Face::South),
            4 => Ok(Face::East),
            5 => Ok(Face::West),
            i => Err(FaceParseError::Unexpected(i)),
        }
    }
}

impl Decode<'_> for UseItemOnS {
    fn decode(r: &mut &'_ [u8]) -> color_eyre::eyre::Result<Self>
    where
        Self: Sized,
    {
        Ok(Self {
            hand: VarInt::decode(r)?.try_into()?,
            location: Position::decode(r)?,
            face: VarInt::decode(r)?.try_into()?,
            cursor_x: r.read_f32::<BigEndian>()?,
            cursor_y: r.read_f32::<BigEndian>()?,
            cursor_z: r.read_f32::<BigEndian>()?,
            inside_block: r.read_u8()? == 1,
            world_border_hit: r.read_u8()? == 1,
            sequence: VarInt::decode(r)?,
        })
    }
}
