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

use std::cmp::Ordering;

use rayon::prelude::*;

use crate::{
    protocol::{
        datatypes::VarInt,
        packets::{
            login::registry::{AllRegistries, Registry},
            play::ChunkDataUpdateLightC,
        },
        Encoder,
    },
    world::World,
};

#[derive(Debug)]
pub struct WorldCache {
    pub encoded: Vec<Vec<u8>>,
}

impl From<World> for WorldCache {
    fn from(world: World) -> Self {
        let mut chunks = world.0.iter().collect::<Vec<_>>();

        chunks.sort_by(|((ax, az), _), ((bx, bz), _)| {
            if (ax + az) > (bx + bz) {
                Ordering::Greater
            } else {
                Ordering::Less
            }
        });

        let chunks = chunks
            .par_iter()
            .map(|(_, c)| ChunkDataUpdateLightC::from(*c))
            .collect::<Vec<ChunkDataUpdateLightC<'_>>>();

        let encoded = chunks
            .par_iter()
            .map(|chunk| {
                let mut encoder = Encoder::new();
                encoder
                    .append_packet(chunk)
                    .expect("Failed to append packet to encoder");
                encoder.take().to_vec()
            })
            .collect();

        Self { encoded }
    }
}

#[derive(Debug)]
pub struct RegistryCache {
    pub encoded: Vec<u8>,
    pub the_end_id: VarInt,
}

impl From<&AllRegistries> for RegistryCache {
    fn from(registry: &AllRegistries) -> Self {
        let mut encoder = Encoder::new();

        let dimensions = Registry::from(registry.dimension_type.clone());
        encoder
            .append_packet(&Registry::from(registry.trim_material.clone()))
            .expect("Failed to encode trim material");
        encoder
            .append_packet(&Registry::from(registry.trim_pattern.clone()))
            .expect("Failed to encode trim pattern");
        encoder
            .append_packet(&Registry::from(registry.banner_pattern.clone()))
            .expect("Failed to encode banner pattern");
        encoder
            .append_packet(&Registry::from(registry.biome.clone()))
            .expect("Failed to encode biome");
        encoder
            .append_packet(&Registry::from(registry.chat_type.clone()))
            .expect("Failed to encode chat type");
        encoder
            .append_packet(&Registry::from(registry.damage_type.clone()))
            .expect("Failed to encode damage type");
        encoder
            .append_packet(&dimensions)
            .expect("Failed to encode dimensions");
        encoder
            .append_packet(&Registry::from(registry.wolf_variant.clone()))
            .expect("Failed to encode wolf variants");
        encoder
            .append_packet(&Registry::from(registry.painting_variant.clone()))
            .expect("Failed to encode painting variants");

        Self {
            encoded: encoder.take().to_vec(),
            the_end_id: VarInt(dimensions.index_of("minecraft:the_end")),
        }
    }
}