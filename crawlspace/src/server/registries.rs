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

use serde::Deserialize;

pub static REGISTRIES: LazyLock<Registries> = LazyLock::new(|| {
    serde_json::from_str(include_str!("../../assets/more_registries.json"))
        .expect("more_registries.json should be parseable")
});

#[derive(Deserialize)]
pub struct Registries {
    #[serde(rename = "minecraft:item")]
    pub item: ItemRegistry,
}

#[derive(Deserialize)]
pub struct ItemRegistry {
    pub default: String,
    pub protocol_id: i32,
    pub entries: HashMap<String, ItemRegistryEntry>,
}

#[derive(Deserialize)]
pub struct ItemRegistryEntry {
    pub protocol_id: i32,
}
