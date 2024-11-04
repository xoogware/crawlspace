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

use fastnbt::SerOpts;
use serde::{Deserialize, Serialize};

use crate::protocol::{datatypes::VarInt, Encode, Packet};

mod banner;
mod biome;
mod chat;
mod damage;
mod dimension;
mod painting;
mod trim;
mod wolf;

pub use banner::*;
pub use biome::*;
pub use chat::*;
pub use damage::*;
pub use dimension::*;
pub use painting::*;
pub use trim::*;
pub use wolf::*;

#[derive(Debug, Serialize, Deserialize)]
pub struct Registry<T: RegistryItem> {
    registry_id: String,
    entries: Vec<RegistryEntry<T>>,
}

impl<T> Packet for Registry<T>
where
    T: RegistryItem,
{
    const ID: i32 = 0x07;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RegistryEntry<T: RegistryItem> {
    id: String,
    entry: Option<T>,
}

impl<T> Registry<T>
where
    T: RegistryItem,
{
    pub fn new(id: &str, entries: Vec<RegistryEntry<T>>) -> Self {
        Self {
            registry_id: id.to_owned(),
            entries,
        }
    }
}

impl<T> Encode for Registry<T>
where
    T: RegistryItem,
{
    fn encode(&self, mut w: impl std::io::Write) -> color_eyre::eyre::Result<()> {
        T::ID.encode(&mut w)?;
        VarInt(self.entries.len() as i32).encode(&mut w)?;

        for RegistryEntry { id, entry } in &self.entries {
            id.encode(&mut w)?;
            match entry {
                None => false.encode(&mut w)?,
                Some(e) => {
                    true.encode(&mut w)?;
                    e.encode(&mut w)?;
                }
            }
        }

        Ok(())
    }
}

// TODO: maybe fill these in? for now we're only using dimensiontype so we can spawn in the end lol
pub trait RegistryItem: Serialize + Sized {
    const ID: &str;

    fn encode(&self, w: impl std::io::Write) -> color_eyre::eyre::Result<()> {
        fastnbt::to_bytes_with_opts(self, SerOpts::network_nbt())?.encode(w)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AllRegistries {
    #[serde(rename = "minecraft:trim_material")]
    trim_material: Registry<TrimMaterial>,
    #[serde(rename = "minecraft:trim_pattern")]
    trim_pattern: Registry<TrimPattern>,
    #[serde(rename = "minecraft:banner_pattern")]
    banner_pattern: Registry<BannerPattern>,
    #[serde(rename = "minecraft:worldgen/biome")]
    biome: Registry<Biome>,
    #[serde(rename = "minecraft:chat_type")]
    chat_type: Registry<ChatType>,
    #[serde(rename = "minecraft:damage_type")]
    damage_type: Registry<DamageType>,
    #[serde(rename = "minecraft:dimension_type")]
    dimension_type: Registry<DimensionType>,
    #[serde(rename = "minecraft:wolf_variant")]
    wolf_variant: Registry<WolfVariant>,
    #[serde(rename = "minecraft:painting_variant")]
    painting_variant: Registry<PaintingVariant>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum StringOrCompound<T> {
    String(String),
    Compound(T),
}
