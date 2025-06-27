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

use std::io::Write;

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use color_eyre::eyre::Result;
use crawlspace_macro::Packet;

use crate::protocol::{Decode, Encode, Packet, PacketDirection, PacketState};

#[derive(Debug, Packet)]
#[packet(
    id = "minecraft:status_request",
    serverbound,
    state = "PacketState::Status"
)]
pub struct StatusRequestS;

impl<'a> Decode<'a> for StatusRequestS {
    fn decode(_buf: &mut &'a [u8]) -> Result<Self> {
        Ok(Self)
    }
}

#[derive(Debug, Packet)]
#[packet(
    id = "minecraft:status_response",
    clientbound,
    state = "PacketState::Status"
)]
pub struct StatusResponseC<'a> {
    pub json_respose: &'a str,
}

impl<'a> Encode for StatusResponseC<'a> {
    fn encode(&self, mut w: impl Write) -> Result<()> {
        self.json_respose.encode(&mut w)
    }
}

#[derive(Debug, Packet)]
#[packet(id = "minecraft:ping", serverbound, state = "PacketState::Status")]
pub struct PingC {
    pub payload: i64,
}

#[derive(Debug, Packet)]
#[packet(id = "minecraft:pong", clientbound, state = "PacketState::Status")]
pub struct PingS {
    pub payload: i64,
}

impl Encode for PingC {
    fn encode(&self, mut w: impl Write) -> Result<()> {
        Ok(w.write_i64::<BigEndian>(self.payload)?)
    }
}

impl<'a> Decode<'a> for PingS {
    fn decode(buf: &mut &'a [u8]) -> Result<Self> {
        Ok(Self {
            payload: buf.read_i64::<BigEndian>()?,
        })
    }
}
