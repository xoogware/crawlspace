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


use color_eyre::eyre::{ensure, Result};

use crate::protocol::{
    datatypes::{Bounded, VarInt},
    Decode, DecodeSized, Encode, Packet,
};

#[derive(Debug)]
pub struct KnownPacksC<'a> {
    known_pack_count: VarInt,
    known_packs: Vec<KnownPack<'a>>,
}

#[derive(Debug)]
pub struct KnownPack<'a> {
    namespace: Bounded<&'a str>,
    id: Bounded<&'a str>,
    version: Bounded<&'a str>,
}

impl Packet for KnownPacksC<'_> {
    const ID: i32 = 0x0E;
}

impl Encode for KnownPacksC<'_> {
    fn encode(&self, mut w: impl std::io::Write) -> Result<()> {
        self.known_pack_count.encode(&mut w)?;
        self.known_packs.encode(&mut w)?;

        Ok(())
    }
}

impl Encode for KnownPack<'_> {
    fn encode(&self, mut w: impl std::io::Write) -> Result<()> {
        self.namespace.encode(&mut w)?;
        self.id.encode(&mut w)?;
        self.version.encode(&mut w)?;

        Ok(())
    }
}

impl<'a> KnownPacksC<'a> {
    pub fn of_version(version: &'a str) -> Self {
        Self {
            known_pack_count: VarInt(1),
            known_packs: vec![KnownPack {
                namespace: Bounded("minecraft"),
                id: Bounded("core"),
                version: Bounded(version),
            }],
        }
    }
}

#[derive(Debug)]
pub struct KnownPacksS<'a> {
    pub known_pack_count: VarInt,
    pub known_packs: Vec<KnownPack<'a>>,
}

impl Packet for KnownPacksS<'_> {
    const ID: i32 = 0x07;
}

impl<'a> Decode<'a> for KnownPacksS<'a> {
    fn decode(r: &mut &'a [u8]) -> Result<Self> {
        let known_pack_count = VarInt::decode(r)?;
        ensure!(known_pack_count.0 >= 0, "Known pack count was less than 0");

        Ok(Self {
            known_packs: Vec::decode(known_pack_count.0 as usize, r)?,
            known_pack_count,
        })
    }
}

impl<'a> Decode<'a> for KnownPack<'a> {
    fn decode(r: &mut &'a [u8]) -> Result<Self> {
        Ok(Self {
            namespace: Bounded::<&'a str>::decode(r)?,
            id: Bounded::<&'a str>::decode(r)?,
            version: Bounded::<&'a str>::decode(r)?,
        })
    }
}

#[derive(Debug)]
pub struct FinishConfigurationC;

impl Packet for FinishConfigurationC {
    const ID: i32 = 0x03;
}

impl Encode for FinishConfigurationC {
    fn encode(&self, _w: impl std::io::Write) -> Result<()> {
        Ok(())
    }
}

#[derive(Debug)]
pub struct FinishConfigurationAckS;

impl Packet for FinishConfigurationAckS {
    const ID: i32 = 0x03;
}

impl<'a> Decode<'a> for FinishConfigurationAckS {
    fn decode(_r: &mut &'a [u8]) -> Result<Self> {
        Ok(Self)
    }
}
