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

use serde::{Deserialize, Serialize};

use super::RegistryItem;

#[derive(Debug, Serialize, Deserialize)]
pub struct DimensionType {
    fixed_time: i64,
    has_skylight: i8,
    has_ceiling: i8,
    ultrawarm: i8,
    natural: i8,
    coordinate_scale: f64,
    bed_works: i8,
    respawn_anchor_works: i8,
    min_y: i32,
    height: i32,
    logical_height: i32,
    infiniburn: String,
    effects: String,
    ambient_light: f32,
    piglin_safe: i8,
    has_raids: i8,
    monster_spawn_light_level: i32,
    monster_spawn_block_light_limit: i32,
}

impl RegistryItem for DimensionType {
    const ID: &str = "minecraft:dimension_type";
}
