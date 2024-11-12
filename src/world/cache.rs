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

use std::{cmp::Ordering, ops::Deref};

use crate::protocol::{packets::play::ChunkDataUpdateLightC, Encode, Encoder};

use super::World;

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
            .iter()
            .map(|(_, c)| ChunkDataUpdateLightC::from(*c))
            .collect::<Vec<ChunkDataUpdateLightC<'_>>>();

        let mut encoder = Encoder::new();
        let mut encoded = Vec::with_capacity(chunks.len());

        chunks.iter().for_each(|chunk| {
            encoder
                .append_packet(chunk)
                .expect("Failed to append packet to encoder");
            encoded.push(encoder.take().to_vec());
        });

        Self { encoded }
    }
}
