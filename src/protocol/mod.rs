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
    mod impls;
    mod position;
    mod slot;
    mod string;
    mod text_component;
    mod variable;

    pub use impls::*;
    pub use position::*;
    pub use slot::*;
    pub use string::*;
    pub use text_component::*;
    pub use variable::*;
}

pub mod packets {
    pub mod login {
        mod config;
        mod handshake;
        #[expect(clippy::module_inception)]
        mod login;
        mod status;

        pub use config::*;
        pub use handshake::*;
        pub use login::*;
        pub use status::*;

        pub mod registry;
    }

    pub mod play {
        mod container;
        mod game_event;
        mod interactions;
        mod keepalive;
        mod login;
        mod position;
        mod status;
        mod teleport;
        mod tick;
        mod world;

        pub use container::*;
        pub use game_event::*;
        pub use interactions::*;
        pub use keepalive::*;
        pub use login::*;
        pub use position::*;
        pub use status::*;
        pub use teleport::*;
        pub use tick::*;
        pub use world::*;
    }
}

mod decoder;
mod encoder;

use std::{fmt::Debug, io::Write};

use bit_vec::BitVec;
use color_eyre::eyre::{Context, Result};
use datatypes::{Bounded, VarInt};
pub use decoder::*;
pub use encoder::*;
use thiserror::Error;

const MAX_PACKET_SIZE: i32 = 2097152;

pub trait Encode {
    fn encode(&self, w: impl Write) -> Result<()>;
}

pub trait Decode<'a> {
    fn decode(r: &mut &'a [u8]) -> Result<Self>
    where
        Self: Sized;
}

pub trait DecodeSized<'a>: Sized {
    fn decode(times: usize, r: &mut &'a [u8]) -> Result<Self>;
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

pub trait Packet {
    const ID: i32;
}

pub trait ServerboundPacket<'a>: Packet + Decode<'a> + Debug {}
impl<'a, P> ServerboundPacket<'a> for P where P: Packet + Decode<'a> + Debug {}

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

#[derive(Debug)]
pub struct Property<'a> {
    name: Bounded<&'a str, 32767>,
    value: Bounded<&'a str, 32767>,
    signature: Option<Bounded<&'a str, 32767>>,
}

impl Encode for Property<'_> {
    fn encode(&self, mut w: impl std::io::Write) -> Result<()> {
        let signed = self.signature.is_some();

        self.name.encode(&mut w)?;
        self.value.encode(&mut w)?;
        signed.encode(&mut w)?;
        self.signature.encode(&mut w)?;

        Ok(())
    }
}

impl Encode for BitVec {
    fn encode(&self, mut w: impl Write) -> Result<()> {
        let mut longs: Vec<i64> = vec![0; (self.len() as f64 / 64.0).ceil() as usize];

        for (i, b) in self.iter().enumerate() {
            longs[i / 64] |= i64::from(b) << (63 - (i % 64))
        }

        VarInt(longs.len() as i32).encode(&mut w)?;

        for long in longs {
            long.encode(&mut w)?;
        }

        Ok(())
    }
}
