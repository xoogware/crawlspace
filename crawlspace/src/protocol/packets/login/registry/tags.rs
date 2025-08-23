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

use crawlspace_macro::Packet;

use crate::protocol::datatypes::{Bounded, VarInt};
use crate::protocol::{Encode, Packet, PacketDirection, PacketState};
use std::collections::HashMap;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, Packet)]
#[packet(
    id = "minecraft:update_tags",
    clientbound,
    state = "PacketState::Configuration"
)]
pub struct AllTags(pub HashMap<String, Tags>);

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Tags(pub HashMap<String, Vec<String>>);

impl Encode for AllTags {
    fn encode(&self, mut w: impl std::io::Write) -> color_eyre::Result<()> {
        VarInt(self.0.len() as i32).encode(&mut w)?;

        for (registry, tags) in self.0.clone() {
            Bounded::<&'_ str>(registry.as_str()).encode(&mut w)?;
            tags.encode(&mut w)?;
        }

        Ok(())
    }
}

impl Encode for Tags {
    fn encode(&self, mut w: impl std::io::Write) -> color_eyre::Result<()> {
        VarInt(self.0.len() as i32).encode(&mut w)?;

        for (name, _) in self.0.clone() {
            Bounded::<&'_ str>(name.as_str()).encode(&mut w)?;
            VarInt(0).encode(&mut w)?;
        }

        Ok(())
    }
}
