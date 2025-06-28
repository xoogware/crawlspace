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

use color_eyre::eyre::Result;
use crawlspace_macro::{Decode, Packet};
use uuid::Uuid;

use crate::protocol::{
    datatypes::{Bounded, Bytes, Rest, VarInt},
    Decode, Encode, Packet, PacketDirection, PacketState, Property,
};

#[derive(Debug, Packet, Decode)]
#[packet(id = "minecraft:hello", serverbound, state = "PacketState::Login")]
pub struct LoginStartS<'a> {
    pub name: Bounded<&'a str, 16>,
    pub player_uuid: Uuid,
}

#[derive(Debug, Packet)]
#[packet(
    id = "minecraft:login_finished",
    clientbound,
    state = "PacketState::Login"
)]
pub struct LoginSuccessC<'a> {
    pub uuid: Uuid,
    pub username: Bounded<&'a str, 16>,
    pub properties: Vec<Property<'a>>,
}

impl<'a> Encode for LoginSuccessC<'a> {
    fn encode(&self, mut w: impl std::io::Write) -> Result<()> {
        let properties_len = VarInt(self.properties.len() as i32);

        self.uuid.encode(&mut w)?;
        self.username.encode(&mut w)?;
        properties_len.encode(&mut w)?;
        self.properties.encode(&mut w)?;

        Ok(())
    }
}

#[derive(Debug, Packet)]
#[packet(
    id = "minecraft:custom_query",
    clientbound,
    state = "PacketState::Login"
)]
pub struct PluginRequestC<'a> {
    pub message_id: VarInt,
    pub channel: Bounded<&'a str, 32767>,
    pub data: Rest<Bytes<'a>, 1048576>,
}

impl<'a> Encode for PluginRequestC<'a> {
    fn encode(&self, mut w: impl std::io::Write) -> Result<()> {
        self.message_id.encode(&mut w)?;
        self.channel.encode(&mut w)?;
        self.data.encode(&mut w)?;

        Ok(())
    }
}

#[derive(Debug, Packet)]
#[packet(
    id = "minecraft:custom_query_answer",
    serverbound,
    state = "PacketState::Login"
)]
pub struct PluginResponseS<'a> {
    pub message_id: VarInt,
    pub data: Option<Rest<Bytes<'a>, 1048576>>,
}

impl<'a> Decode<'a> for PluginResponseS<'a> {
    fn decode(r: &mut &'a [u8]) -> Result<Self> {
        Ok(Self {
            message_id: VarInt::decode(r)?,
            data: if bool::decode(r)? {
                Some(Rest::<Bytes<'a>, 1048576>::decode(r)?)
            } else {
                None
            },
        })
    }
}

#[derive(Debug, Packet, Decode)]
#[packet(
    id = "minecraft:login_acknowledged",
    serverbound,
    state = "PacketState::Login"
)]
pub struct LoginAckS;
