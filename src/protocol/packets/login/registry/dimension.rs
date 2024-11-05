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

use super::{deserialize_bool, RegistryItem};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DimensionType {
    fixed_time: Option<i64>,
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
    monster_spawn_light_level: IntOrLightLevel,
    monster_spawn_block_light_limit: i32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum IntOrLightLevel {
    Int(i32),
    LL(IntProvider),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
enum IntProvider {
    #[serde(rename = "minecraft:constant")]
    Constant { value: i32 },
    #[serde(rename = "minecraft:uniform")]
    Uniform {
        min_inclusive: i32,
        max_inclusive: i32,
    },
    #[serde(rename = "minecraft:biased_to_bottom")]
    BiasedToBottom {
        min_inclusive: i32,
        max_inclusive: i32,
    },
    #[serde(rename = "minecraft:clamped")]
    Clamped {
        min_inclusive: i32,
        max_inclusive: i32,
        source: Box<IntProvider>,
    },
    #[serde(rename = "minecraft:clamped_normal")]
    ClampedNormal {
        mean: f32,
        deviation: f32,
        min_inclusive: i32,
        max_inclusive: i32,
    },
    #[serde(rename = "minecraft:weighted_list")]
    WeightedList { distribution: Vec<WeightedProvider> },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct WeightedProvider {
    data: IntProvider,
    weight: i32,
}

impl RegistryItem for DimensionType {
    const ID: &str = "minecraft:dimension_type";
}
