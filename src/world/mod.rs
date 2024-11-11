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

use std::{collections::HashMap, fs::File};

use color_eyre::eyre::Result;
use fastanvil::Region;
use serde::Deserialize;

pub mod blocks;

#[derive(Clone, Debug)]
pub struct World(pub HashMap<(i32, i32), Chunk>);

#[derive(Clone, Debug, Deserialize)]
pub struct Chunk {
    #[serde(rename = "DataVersion")]
    pub data_version: i32,
    #[serde(rename = "xPos")]
    pub x_pos: i32,
    #[serde(rename = "zPos")]
    pub z_pos: i32,
    #[serde(rename = "yPos")]
    pub y_pos: i32,
    #[serde(rename = "Status")]
    pub status: ChunkStatus,
    #[serde(rename = "LastUpdate")]
    pub last_update: f64,
    pub sections: Vec<Section>,
}

#[derive(Clone, Debug, Deserialize)]
pub enum ChunkStatus {
    #[serde(rename = "minecraft:empty")]
    Empty,
    #[serde(rename = "minecraft:structure_starts")]
    StructureStarts,
    #[serde(rename = "minecraft:structure_references")]
    StructureReferences,
    #[serde(rename = "minecraft:biomes")]
    Biomes,
    #[serde(rename = "minecraft:noise")]
    Noise,
    #[serde(rename = "minecraft:surface")]
    Surface,
    #[serde(rename = "minecraft:carvers")]
    Carvers,
    #[serde(rename = "minecraft:features")]
    Features,
    #[serde(rename = "minecraft:light")]
    Light,
    #[serde(rename = "minecraft:initialize_light")]
    InitializeLight,
    #[serde(rename = "minecraft:spawn")]
    Spawn,
    #[serde(rename = "minecraft:full")]
    Full,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Section {
    #[serde(rename = "Y")]
    pub y: i32,
    pub block_states: BlockStates,
    pub biomes: Biomes,
    #[serde(rename = "BlockLight")]
    pub block_light: Option<fastnbt::ByteArray>,
    #[serde(rename = "SkyLight")]
    pub sky_light: Option<fastnbt::ByteArray>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct BlockStates {
    pub palette: Vec<Block>,
    pub data: Option<fastnbt::LongArray>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Block {
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "Properties", default)]
    pub properties: HashMap<String, String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Biomes {
    pub palette: Vec<String>,
    pub data: Option<fastnbt::LongArray>,
}

pub fn read_world(path: &str) -> Result<World> {
    let folder = std::fs::read_dir(path).unwrap();
    let mut chunks = World(HashMap::new());

    for path in folder {
        let file = File::open(path.unwrap().path())?;
        let mut region = Region::from_stream(file)?;

        for chunk in region.iter() {
            let chunk = chunk.unwrap();
            let mut parsed: Chunk = fastnbt::from_bytes(&chunk.data).unwrap_or_else(|e| {
                panic!(
                    "Failed to parse chunk {e}: {}",
                    &chunk
                        .data
                        .iter()
                        .map(|b| b.to_string())
                        .collect::<Vec<String>>()
                        .join(" ")
                );
            });

            if (-10..10).contains(&parsed.x_pos) && (-10..10).contains(&parsed.z_pos) {
                parsed.sections.sort_by_key(|c| c.y);

                debug!(
                    "Successfully parsed chunk at {}, {}",
                    parsed.x_pos, parsed.z_pos
                );
                trace!("{:?}", parsed);

                chunks.0.insert((parsed.x_pos, parsed.z_pos), parsed);
            }
        }
    }

    Ok(chunks)
}
