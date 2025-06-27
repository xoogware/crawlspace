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

use std::{collections::HashMap, sync::LazyLock};

use crawlspace_macro::Packet;
use fastnbt::SerOpts;
use serde::{de, Deserialize, Serialize};

use crate::protocol::{datatypes::VarInt, Encode, Packet, PacketDirection, PacketState};

mod banner;
mod biome;
mod chat;
mod damage;
mod dimension;
mod painting;
mod tags;
mod trim;
mod wolf;

pub use banner::*;
pub use biome::*;
pub use chat::*;
pub use damage::*;
pub use dimension::*;
pub use painting::*;
pub use tags::*;
pub use trim::*;
pub use wolf::*;

pub static ALL_REGISTRIES: LazyLock<AllRegistries> = LazyLock::new(|| {
    serde_json::from_str(include_str!("../../../../../assets/registries.json"))
        .expect("registries.json should be parseable")
});

pub static TAGS: LazyLock<AllTags> = LazyLock::new(|| {
    serde_json::from_str(include_str!("../../../../../assets/tags.json"))
        .expect("tags.json should be parseable")
});

#[derive(Clone, Debug, Serialize, Deserialize, Packet)]
#[packet(
    id = "minecraft:registry_data",
    clientbound,
    state = "PacketState::Configuration"
)]
pub struct Registry<T>
where
    T: RegistryItem,
{
    registry_id: String,
    pub entries: Vec<RegistryEntry<T>>,
}

impl<T: RegistryItem> Registry<T> {
    pub fn index_of(&self, id: &str) -> i32 {
        self.entries
            .iter()
            .position(|e| e.id == id)
            .unwrap_or_else(|| panic!("Element {id} should be in registry {}!", self.registry_id))
            .try_into()
            .unwrap()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RegistryEntry<T: RegistryItem> {
    id: String,
    entry: Option<T>,
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
pub trait RegistryItem: Serialize + Sized + Clone {
    const ID: &str;

    fn encode(&self, w: impl std::io::Write) -> color_eyre::eyre::Result<()> {
        fastnbt::to_bytes_with_opts(self, SerOpts::network_nbt())?.encode(w)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AllRegistries {
    #[serde(rename = "minecraft:trim_material")]
    pub trim_material: HashMap<String, TrimMaterial>,
    #[serde(rename = "minecraft:trim_pattern")]
    pub trim_pattern: HashMap<String, TrimPattern>,
    #[serde(rename = "minecraft:banner_pattern")]
    pub banner_pattern: HashMap<String, BannerPattern>,
    #[serde(rename = "minecraft:worldgen/biome")]
    pub biome: HashMap<String, Biome>,
    #[serde(rename = "minecraft:chat_type")]
    pub chat_type: HashMap<String, ChatType>,
    #[serde(rename = "minecraft:damage_type")]
    pub damage_type: HashMap<String, DamageType>,
    #[serde(rename = "minecraft:dimension_type")]
    pub dimension_type: HashMap<String, DimensionType>,
    #[serde(rename = "minecraft:wolf_variant")]
    pub wolf_variant: HashMap<String, WolfVariant>,
    #[serde(rename = "minecraft:painting_variant")]
    pub painting_variant: HashMap<String, PaintingVariant>,
}

impl<T: RegistryItem> From<HashMap<String, T>> for Registry<T> {
    fn from(value: HashMap<String, T>) -> Self {
        let entries = value
            .into_iter()
            .map(|(k, v)| RegistryEntry {
                id: k,
                entry: Some(v),
            })
            .collect();

        Self {
            registry_id: T::ID.to_owned(),
            entries,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum StringOrCompound<T> {
    String(String),
    Compound(T),
}

fn deserialize_bool<'de, D>(deserializer: D) -> Result<i8, D::Error>
where
    D: de::Deserializer<'de>,
{
    let b: bool = de::Deserialize::deserialize(deserializer)?;

    match b {
        true => Ok(1),
        false => Ok(0),
    }
}
