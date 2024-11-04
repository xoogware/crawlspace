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

use std::collections::HashMap;

use fastnbt::SerOpts;
use serde::Serialize;

use crate::protocol::{datatypes::VarInt, Encode, Packet};

#[derive(Debug)]
pub struct Registry<T: RegistryItem>(HashMap<String, Option<T>>);

impl<T> Registry<T>
where
    T: RegistryItem,
{
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn insert(&mut self, ident: &str, item: Option<T>) {
        self.0.insert(ident.to_owned(), item);
    }
}

impl<T> Packet for Registry<T>
where
    T: RegistryItem,
{
    const ID: i32 = 0x07;
}

impl<T> Encode for Registry<T>
where
    T: RegistryItem,
{
    fn encode(&self, mut w: impl std::io::Write) -> color_eyre::eyre::Result<()> {
        T::ID.encode(&mut w)?;
        VarInt(self.0.len() as i32).encode(&mut w)?;

        for (ident, entry) in &self.0 {
            ident.encode(&mut w)?;
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
pub trait RegistryItem: Encode {
    const ID: &str;
}

#[derive(Debug, Serialize)]
pub struct DimensionType<'a> {
    fixed_time: Option<i64>,
    has_skylight: bool,
    has_ceiling: bool,
    ultrawarm: bool,
    natural: bool,
    coordinate_scale: f64,
    bed_works: bool,
    respawn_anchor_works: bool,
    min_y: i32,
    height: i32,
    logical_height: i32,
    infiniburn: &'a str,
    effects: &'a str,
    ambient_light: f32,
    piglin_safe: bool,
    has_raids: bool,
    monster_spawn_light_level: i32,
    monster_spawn_block_light_limit: i32,
}

impl DimensionType<'static> {
    pub const fn the_end() -> Self {
        Self {
            fixed_time: Some(6000),
            has_skylight: false,
            has_ceiling: false,
            ultrawarm: false,
            natural: false,
            coordinate_scale: 1.0,
            bed_works: false,
            respawn_anchor_works: false,
            min_y: 0,
            height: 256,
            logical_height: 256,
            infiniburn: "#minecraft:infiniburn_end",
            effects: "minecraft:the_end",
            ambient_light: 10.0,
            piglin_safe: false,
            has_raids: false,
            monster_spawn_light_level: 0,
            monster_spawn_block_light_limit: 0,
        }
    }
}

impl RegistryItem for DimensionType<'_> {
    const ID: &'static str = "minecraft:dimension_type";
}

impl Encode for DimensionType<'_> {
    fn encode(&self, w: impl std::io::Write) -> color_eyre::eyre::Result<()> {
        debug!("serializing {:#?}", self);
        fastnbt::to_bytes_with_opts(self, SerOpts::new().serialize_root_compound_name(false))?
            .encode(w)
    }
}
