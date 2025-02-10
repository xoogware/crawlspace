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

use super::{deserialize_bool, RegistryItem, StringOrCompound};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Biome {
    #[serde(deserialize_with = "deserialize_bool")]
    has_precipitation: i8,
    temperature: f32,
    temperature_modifier: Option<String>,
    downfall: f32,
    effects: Effects,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Effects {
    fog_color: i32,
    water_color: i32,
    water_fog_color: i32,
    sky_color: i32,
    foliage_color: Option<i32>,
    grass_color: Option<i32>,
    grass_color_modifier: Option<String>,
    particle: Option<Particle>,
    ambient_sound: Option<StringOrCompound<AmbientSound>>,
    mood_sound: Option<MoodSound>,
    additions_sound: Option<AdditionsSound>,
    music: Option<Vec<Music>>,
    music_volume: Option<f32>
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Particle {
    options: ParticleOptions,
    probability: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct ParticleOptions {
    #[serde(rename = "type")]
    kind: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct AmbientSound {
    sound_id: String,
    range: Option<f32>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct MoodSound {
    sound: String,
    tick_delay: i32,
    block_search_extent: i32,
    offset: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct AdditionsSound {
    sound: String,
    tick_chance: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Music {
    data: MusicData,
    weight: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct MusicData {
    sound: String,
    min_delay: i32,
    max_delay: i32,
    #[serde(deserialize_with = "deserialize_bool")]
    replace_current_music: i8,
}

impl RegistryItem for Biome {
    const ID: &str = "minecraft:worldgen/biome";
}
