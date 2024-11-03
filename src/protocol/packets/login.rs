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

use std::borrow::Cow;

use color_eyre::eyre::Result;
use uuid::Uuid;

use crate::protocol::{
    datatypes::{Bounded, Bytes, Ident, VarInt},
    ClientboundPacket, Decode, Encode, Packet,
};

#[derive(Debug)]
pub struct LoginStartS<'a> {
    pub name: Bounded<&'a str, 16>,
    pub player_uuid: Uuid,
}

impl Packet for LoginStartS<'_> {
    const ID: i32 = 0x00;
}

impl<'a> Decode<'a> for LoginStartS<'a> {
    fn decode(buf: &mut &'a [u8]) -> Result<Self> {
        Ok(Self {
            name: Bounded::<&'a str, 16>::decode(buf)?,
            player_uuid: Uuid::decode(buf)?,
        })
    }
}

#[derive(Debug)]
pub struct LoginSuccessC<'a> {
    pub uuid: Uuid,
    pub username: Bounded<&'a str, 16>,
    pub properties: Vec<Property<'a>>,
    pub strict_error_handling: bool,
}

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

impl Packet for LoginSuccessC<'_> {
    const ID: i32 = 0x02;
}

impl<'a> Encode for LoginSuccessC<'a> {
    fn encode(&self, mut w: impl std::io::Write) -> Result<()> {
        let properties_len = VarInt(self.properties.len() as i32);

        self.uuid.encode(&mut w)?;
        self.username.encode(&mut w)?;
        properties_len.encode(&mut w)?;
        self.properties.encode(&mut w)?;
        self.strict_error_handling.encode(&mut w)?;

        Ok(())
    }
}

#[derive(Debug)]
pub struct PluginRequestC<'a> {
    pub message_id: VarInt,
    pub channel: Bounded<&'a str, 32767>,
    pub data: Bounded<Bytes<'a>, 1048576>,
}

impl Packet for PluginRequestC<'_> {
    const ID: i32 = 0x04;
}

impl<'a> Encode for PluginRequestC<'a> {
    fn encode(&self, mut w: impl std::io::Write) -> Result<()> {
        self.message_id.encode(&mut w)?;
        self.channel.encode(&mut w)?;
        self.data.encode(&mut w)?;

        Ok(())
    }
}

#[derive(Debug)]
pub struct LoginAckS;

impl Packet for LoginAckS {
    const ID: i32 = 0x03;
}

impl Decode<'_> for LoginAckS {
    fn decode(_r: &mut &'_ [u8]) -> Result<Self> {
        Ok(Self)
    }
}
