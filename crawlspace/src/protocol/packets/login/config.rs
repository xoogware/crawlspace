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
use crawlspace_macro::{Decode, Encode, Packet};

use crate::protocol::{
    datatypes::{Bounded, VarInt},
    Decode, DecodeSized, Encode, Packet, PacketDirection, PacketState,
};

#[derive(Debug, Packet, Encode)]
#[packet(
    id = "minecraft:select_known_packs",
    clientbound,
    state = "PacketState::Configuration"
)]
pub struct KnownPacksC<'a> {
    known_pack_count: VarInt,
    known_packs: Vec<KnownPack<'a>>,
}

#[derive(Debug, Encode, Decode)]
pub struct KnownPack<'a> {
    namespace: Bounded<&'a str>,
    id: Bounded<&'a str>,
    version: Bounded<&'a str>,
}

impl<'a> KnownPacksC<'a> {
    pub fn of_version(version: &'a str) -> Self {
        Self {
            known_pack_count: VarInt(2),
            known_packs: vec![
                KnownPack {
                    namespace: Bounded("minecraft"),
                    id: Bounded("core"),
                    version: Bounded(version),
                },
                KnownPack {
                    namespace: Bounded("minecraft"),
                    id: Bounded("root"),
                    version: Bounded(version),
                },
            ],
        }
    }
}

#[derive(Debug, Packet)]
#[packet(
    id = "minecraft:select_known_packs",
    serverbound,
    state = "PacketState::Configuration"
)]
pub struct KnownPacksS<'a> {
    pub _known_pack_count: VarInt,
    pub _known_packs: Vec<KnownPack<'a>>,
}

impl<'a> Decode<'a> for KnownPacksS<'a> {
    fn decode(r: &mut &'a [u8]) -> Result<Self> {
        let known_pack_count = VarInt::decode(r)?;
        ensure!(known_pack_count.0 >= 0, "Known pack count was less than 0");

        Ok(Self {
            _known_packs: Vec::decode(known_pack_count.0 as usize, r)?,
            _known_pack_count: known_pack_count,
        })
    }
}

#[derive(Debug, Packet, Encode)]
#[packet(
    id = "minecraft:finish_configuration",
    clientbound,
    state = "PacketState::Configuration"
)]
pub struct FinishConfigurationC;

#[derive(Debug, Packet, Decode)]
#[packet(
    id = "minecraft:finish_configuration",
    serverbound,
    state = "PacketState::Configuration"
)]
pub struct FinishConfigurationAckS;
