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

use bit_vec::BitVec;
use color_eyre::eyre::{bail, Context, Result};
use datatypes::{Bounded, VarInt};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::LazyLock;
use std::{fmt::Debug, io::Write};
use thiserror::Error;

pub use decoder::*;
pub use encoder::*;

pub static PACKETS: LazyLock<Packets> = LazyLock::new(|| {
    Packets::new(
        serde_json::from_str(include_str!("../../assets/packets.json"))
            .expect("packets.json should be parseable"),
    )
});

#[derive(Clone, Debug, Deserialize)]
pub struct PacketType {
    pub protocol_id: i32,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ForwardPackets(
    pub HashMap<PacketState, HashMap<PacketDirection, HashMap<String, PacketType>>>,
);

#[derive(Clone, Debug)]
pub struct Packets {
    forward: ForwardPackets,
    reverse: HashMap<PacketState, HashMap<PacketDirection, HashMap<i32, String>>>,
}

impl Packets {
    pub fn new(forward: ForwardPackets) -> Self {
        Self {
            reverse: Self::build_reverse(&forward),
            forward,
        }
    }

    pub fn get_protocol_id(
        &self,
        state: PacketState,
        direction: PacketDirection,
        name: &str,
    ) -> Option<i32> {
        Some(
            self.forward
                .0
                .get(&state)?
                .get(&direction)?
                .get(name)?
                .protocol_id,
        )
    }

    pub fn get_resource_id(
        &self,
        state: PacketState,
        direction: PacketDirection,
        protocol_id: i32,
    ) -> Option<&String> {
        self.reverse.get(&state)?.get(&direction)?.get(&protocol_id)
    }

    fn build_reverse(
        forward: &ForwardPackets,
    ) -> HashMap<PacketState, HashMap<PacketDirection, HashMap<i32, String>>> {
        let mut reverse = HashMap::new();

        for state in forward.0.keys() {
            let direction_mapping = reverse
                .entry(*state)
                .or_insert_with(HashMap::new)
                .entry(PacketDirection::Serverbound)
                .or_insert_with(HashMap::new);

            for (key, value) in forward
                .0
                .get(state)
                .unwrap()
                .get(&PacketDirection::Serverbound)
                .unwrap()
            {
                direction_mapping
                    .entry(value.protocol_id)
                    .or_insert_with(move || key.clone());
            }
        }

        reverse
    }
}

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

#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PacketDirection {
    Clientbound,
    Serverbound,
}

#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PacketState {
    Handshake,
    Login,
    Configuration,
    Play,
    Status,
}

#[derive(Clone, Copy, Debug)]
pub enum ProtocolState {
    Handshaking,
    Play,
    Status,
    Login,
    Transfer,
}

impl Decode<'_> for ProtocolState {
    fn decode(r: &mut &'_ [u8]) -> Result<Self>
    where
        Self: Sized,
    {
        let state = VarInt::decode(r)?;

        match state.0 {
            1 => Ok(ProtocolState::Status),
            2 => Ok(ProtocolState::Login),
            3 => Ok(ProtocolState::Transfer),
            e => bail!("Invalid next state requested: {e}"),
        }
    }
}

#[derive(Error, Debug)]
pub enum ProtocolStateDecodeError {
    #[error("Unable to decode {0} into a PacketState")]
    InvalidState(i32),
}

impl From<ProtocolState> for PacketState {
    fn from(value: ProtocolState) -> Self {
        match value {
            ProtocolState::Handshaking => PacketState::Handshake,
            ProtocolState::Play => PacketState::Play,
            ProtocolState::Status => PacketState::Status,
            ProtocolState::Login => PacketState::Login,
            ProtocolState::Transfer => PacketState::Login,
        }
    }
}

impl TryFrom<i32> for ProtocolState {
    type Error = ProtocolStateDecodeError;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(ProtocolState::Status),
            2 => Ok(ProtocolState::Login),
            3 => Ok(ProtocolState::Transfer),
            i => Err(ProtocolStateDecodeError::InvalidState(i)),
        }
    }
}

pub trait Packet {
    fn id() -> &'static str;
    fn state() -> PacketState;
    fn direction() -> PacketDirection;

    fn get_id() -> i32 {
        PACKETS
            .get_protocol_id(Self::state(), Self::direction(), Self::id())
            .unwrap_or_else(|| {
                panic!(
                    "expected packet {:?}/{} ({:?}) to exist",
                    Self::state(),
                    Self::id(),
                    Self::direction()
                )
            })
    }
}

pub trait ServerboundPacket<'a>: Packet + Decode<'a> + Debug {
    const DIRECTION: PacketDirection = PacketDirection::Serverbound;
}
impl<'a, P> ServerboundPacket<'a> for P where P: Packet + Decode<'a> + Debug {}

pub trait ClientboundPacket: Packet + Encode + Debug {
    fn encode_packet(&self, mut w: impl Write) -> Result<()>
    where
        Self: Encode,
    {
        VarInt(Self::get_id())
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
