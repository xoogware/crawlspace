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

pub mod datatypes {
    mod string;
    mod variable;

    pub use string::*;
    pub use variable::*;
}

pub mod packets {
    mod handshake;
    mod status;

    pub use handshake::*;
    pub use status::*;
}

mod decoder;
mod encoder;

use std::{fmt::Debug, io::Write};

use bytes::BytesMut;
use color_eyre::eyre::{bail, Context, Result};
use datatypes::VarInt;
pub use decoder::*;
pub use encoder::*;
use thiserror::Error;

const MAX_PACKET_SIZE: i32 = 2097152;

pub trait Encode {
    fn encode(&self, w: impl Write) -> Result<()>;
}

pub trait Decode<'a>: Sized {
    fn decode(r: &mut &'a [u8]) -> Result<Self>;
}

pub enum PacketDirection {
    Clientbound,
    Serverbound,
}

#[derive(Clone, Copy, Debug)]
pub enum PacketState {
    Handshaking,
    Play,
    Status,
    Login,
    Transfer,
}

#[derive(Error, Debug)]
pub enum PacketStateDecodeError {
    #[error("Unable to decode {0} into a PacketState")]
    InvalidState(i32),
}

impl TryFrom<i32> for PacketState {
    type Error = PacketStateDecodeError;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(PacketState::Status),
            2 => Ok(PacketState::Login),
            3 => Ok(PacketState::Transfer),
            i => Err(PacketStateDecodeError::InvalidState(i)),
        }
    }
}

trait Packet {
    const ID: i32;
}

#[expect(private_bounds)]
pub trait ServerboundPacket<'a>: Packet + Decode<'a> + Debug {}
impl<'a, P> ServerboundPacket<'a> for P where P: Packet + Decode<'a> + Debug {}

#[expect(private_bounds)]
pub trait ClientboundPacket: Packet + Encode + Debug {
    fn encode_packet(&self, mut w: impl Write) -> Result<()>
    where
        Self: Encode,
    {
        VarInt(Self::ID)
            .encode(&mut w)
            .context("Failed to encode packet id")?;

        self.encode(w)
    }
}
impl<P> ClientboundPacket for P where P: Packet + Encode + Debug {}
