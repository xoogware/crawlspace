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

use std::{cmp::Ordering, collections::HashMap};

use rayon::prelude::*;

use crate::{
    protocol::{
        datatypes::VarInt,
        packets::{
            login::registry::{AllRegistries, Registry, AllTags},
            play::ChunkDataUpdateLightC,
        },
        Encoder,
    },
    world::{blocks::Blocks, BlockEntity, Container, World},
    CrawlState,
};
use crate::protocol::packets::login::registry::RegistryItem;

#[derive(Debug)]
pub struct WorldCache {
    pub encoded: Vec<Vec<u8>>,
    pub containers: HashMap<(i32, i32, i32), Container>,
}

impl WorldCache {
    pub fn from_anvil(crawlstate: CrawlState, world: &World) -> Self {
        let mut chunks = world.0.iter().collect::<Vec<_>>();

        chunks.sort_by(|((ax, az), _), ((bx, bz), _)| {
            if (ax + az) > (bx + bz) {
                Ordering::Greater
            } else {
                Ordering::Less
            }
        });

        let block_states = Blocks::new();

        let containers = chunks
            .iter()
            .map(|(_, c)| {
                c.block_entities
                    .iter()
                    .filter_map(|block_entity| {
                        // TODO: cache this somewhere so block entities aren't parsed twice on startup
                        let block_entity = BlockEntity::try_parse((*block_entity).clone())
                            .map_or_else(
                                |why| {
                                    warn!(
                                        "Failed to parse block entity: {why}, ignoring in container cache for ({}, {})",
                                        c.x_pos,
                                        c.z_pos,
                                    );
                                    None
                                },
                                |e| match e.keep_packed {
                                    true => None,
                                    false => Some(e),
                                },
                            );

                        let Some(block_entity) = block_entity else {
                            return None;
                        };

                        match block_entity.id.as_str() {
                            "minecraft:chest" | "minecraft:trapped_chest" | "minecraft:barrel" => {
                                Some(block_entity)
                            }
                            _ => None,
                        }
                    })
                    .map(|container| {
                        (
                            (container.x, container.y, container.z),
                            Container::try_from(container).expect("Failed to convert container from block entity NBT"),
                        )
                    })
                    .collect::<Vec<((i32, i32, i32), Container)>>()
            })
            .flatten()
            .collect();

        debug!("Containers: {:?}", containers);

        let encoded = chunks
            .par_iter()
            .map(|(_, chunk)| {
                let mut encoder = Encoder::new();
                encoder
                    .append_packet(&ChunkDataUpdateLightC::new(
                        crawlstate.clone(),
                        chunk,
                        &block_states,
                    ))
                    .expect("Failed to append packet to encoder");
                encoder.take().to_vec()
            })
            .collect();

        Self {
            encoded,
            containers,
        }
    }
}

#[derive(Debug)]
pub struct TagCache {
    pub encoded: Vec<u8>,
}

impl From<&AllTags> for TagCache {
    fn from(tags: &AllTags) -> Self {
        let mut encoder = Encoder::new();

        encoder
            .append_packet(tags)
            .expect("Failed to encode tags");

        Self {
            encoded: encoder.take().to_vec()
        }
    }
}

#[derive(Debug)]
pub struct RegistryCache {
    pub encoded: Vec<u8>,
    pub the_end_id: VarInt,
    pub the_end_biome_id: u16,
}

impl From<&AllRegistries> for RegistryCache {
    fn from(registry: &AllRegistries) -> Self {
        let mut encoder = Encoder::new();

        let dimensions = Registry::from(registry.dimension_type.clone());
        let biomes = Registry::from(registry.biome.clone());
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
            .append_packet(&Registry::from(registry.chat_type.clone()))
            .expect("Failed to encode chat type");
        encoder
            .append_packet(&Registry::from(registry.damage_type.clone()))
            .expect("Failed to encode damage type");
        encoder
            .append_packet(&dimensions)
            .expect("Failed to encode dimensions");
        encoder
            .append_packet(&biomes)
            .expect("Failed to encode biomes");
        encoder
            .append_packet(&Registry::from(registry.wolf_variant.clone()))
            .expect("Failed to encode wolf variants");
        encoder
            .append_packet(&Registry::from(registry.painting_variant.clone()))
            .expect("Failed to encode painting variants");

        Self {
            encoded: encoder.take().to_vec(),
            the_end_id: VarInt(dimensions.index_of("minecraft:the_end")),
            the_end_biome_id: biomes.index_of("minecraft:the_end") as u16,
        }
    }
}
