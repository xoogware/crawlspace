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

use bit_vec::BitVec;
use bytes::{BufMut, BytesMut};
use fastnbt::SerOpts;
use serde::Serialize;

use crate::{
    protocol::{datatypes::VarInt, packets::login::registry::Registry, Encode, Packet},
    world::{
        self,
        blocks::{BlockState, Blocks, ALL_BLOCKS},
    },
};

#[derive(Debug)]
pub struct SetCenterChunkC {
    pub x: VarInt,
    pub y: VarInt,
}

impl Packet for SetCenterChunkC {
    const ID: i32 = 0x54;
}

impl Encode for SetCenterChunkC {
    fn encode(&self, mut w: impl std::io::Write) -> color_eyre::eyre::Result<()> {
        self.x.encode(&mut w)?;
        self.y.encode(&mut w)?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct ChunkDataUpdateLightC<'a> {
    x: i32,
    z: i32,
    /// Currently unused (no snow/rain/beacons anyway)
    heightmaps: HeightMaps,
    data: Vec<ChunkSection>,
    entities: Vec<BlockEntity<'a>>,
    sky_light_mask: BitVec,
    block_light_mask: BitVec,
    empty_sky_light_mask: BitVec,
    empty_block_light_mask: BitVec,
    sky_light_arrays: Vec<&'a [u8]>,
    block_light_arrays: Vec<&'a [u8]>,
}

#[derive(Debug)]
#[expect(unused)]
struct BlockEntity<'a> {
    packed_xz: u8,
    y: i16,
    kind: VarInt,
    data: &'a [u8],
}

impl Encode for BlockEntity<'_> {
    fn encode(&self, _w: impl std::io::Write) -> color_eyre::eyre::Result<()> {
        unimplemented!("Block entities are currently unsupported");
    }
}

#[derive(Debug)]
struct HeightMaps(HashMap<String, fastnbt::LongArray>);

#[derive(Debug)]
struct ChunkSection {
    block_count: i16,
    block_states: PalettedContainer,
    biomes: PalettedContainer,
}

impl Encode for ChunkSection {
    fn encode(&self, mut w: impl std::io::Write) -> color_eyre::eyre::Result<()> {
        self.block_count.encode(&mut w)?;
        self.block_states.encode(&mut w)?;
        self.biomes.encode(&mut w)?;

        Ok(())
    }
}

#[derive(Debug)]
struct PalettedContainer {
    bits_per_entry: u8,
    palette: Palette,
    data_array: fastnbt::LongArray,
}

#[derive(Debug)]
enum Palette {
    SingleValued(BlockState),
    Indirect(VarInt, Vec<BlockState>),
    Direct,
}

impl Encode for PalettedContainer {
    fn encode(&self, mut w: impl std::io::Write) -> color_eyre::eyre::Result<()> {
        self.bits_per_entry.encode(&mut w)?;

        match &self.palette {
            Palette::SingleValued(v) => {
                VarInt(v.0 as i32).encode(&mut w)?;
            }
            Palette::Indirect(len, palette) => {
                len.encode(&mut w)?;
                for id in palette {
                    VarInt(id.0 as i32).encode(&mut w)?;
                }
            }
            Palette::Direct => (),
        }

        VarInt(self.data_array.len() as i32).encode(&mut w)?;

        for long in self.data_array.iter() {
            long.encode(&mut w)?;
        }

        Ok(())
    }
}

impl Packet for ChunkDataUpdateLightC<'_> {
    const ID: i32 = 0x27;
}

impl Encode for ChunkDataUpdateLightC<'_> {
    fn encode(&self, mut w: impl std::io::Write) -> color_eyre::eyre::Result<()> {
        self.x.encode(&mut w)?;
        self.z.encode(&mut w)?;

        let heightmaps = fastnbt::to_bytes_with_opts(&self.heightmaps.0, SerOpts::network_nbt())?;
        heightmaps.encode(&mut w)?;

        let mut chunk_buf = Vec::new().writer();

        for chunk in &self.data {
            chunk.encode(&mut chunk_buf)?;
        }

        VarInt(chunk_buf.get_ref().len() as i32).encode(&mut w)?;
        chunk_buf.get_ref().encode(&mut w)?;

        VarInt(self.entities.len() as i32).encode(&mut w)?;
        for e in &self.entities {
            e.encode(&mut w)?;
        }

        self.sky_light_mask.encode(&mut w)?;
        self.block_light_mask.encode(&mut w)?;
        self.empty_sky_light_mask.encode(&mut w)?;
        self.empty_block_light_mask.encode(&mut w)?;

        VarInt(self.sky_light_arrays.len() as i32).encode(&mut w)?;
        for sky_light_array in &self.sky_light_arrays {
            VarInt(sky_light_array.len() as i32).encode(&mut w)?;
            w.write_all(sky_light_array)?;
        }

        VarInt(self.block_light_arrays.len() as i32).encode(&mut w)?;
        for block_light_array in &self.block_light_arrays {
            VarInt(block_light_array.len() as i32).encode(&mut w)?;
            w.write_all(block_light_array)?;
        }

        Ok(())
    }
}

impl ChunkSection {
    pub fn anvil_to_sec(value: &world::Section) -> Self {
        let mut blocks: [i16; 16 * 16 * 16] = [0; 16 * 16 * 16];
        let bit_length = (64 - (value.block_states.palette.len() as u64).leading_zeros()).max(4);
        let blocks_per_long = 64 / bit_length;

        let bit_mask = (1 << bit_length) - 1;

        match value.block_states.data {
            None => blocks.fill(0),
            Some(ref data) => {
                let mut i = 0;
                for long in data.iter() {
                    let long = *long as u64;
                    for b in 0..blocks_per_long {
                        blocks[i] = ((long >> (bit_length * b)) & bit_mask) as i16;
                        i += 1;
                    }
                }
            }
        }

        let palette = value
            .block_states
            .palette
            .iter()
            .map(|b| BlockState::try_from(b).unwrap_or(BlockState::AIR))
            .collect::<Vec<_>>();

        let blocks: Vec<u16> = blocks
            .iter()
            .map(|b| palette.get(*b as usize).unwrap().0)
            .collect();

        let block_count = blocks.iter().filter(|b| **b != 0).collect::<Vec<_>>().len();

        let bit_length = match palette.len() {
            1 => 0,
            l => (64 - l.leading_zeros()).max(4) as u8,
        };

        let palette = {
            if bit_length == 15 {
                Palette::Direct
            } else if bit_length >= 4 {
                Palette::Indirect(VarInt(palette.len() as i32), palette)
            } else {
                Palette::SingleValued(*palette.first().unwrap())
            }
        };

        let blocks = match palette {
            Palette::Indirect(_, ref p) => blocks
                .iter()
                .map(|requested| p.iter().position(|pb| pb.0 == *requested).unwrap() as u16)
                .collect::<Vec<_>>(),
            _ => blocks,
        };

        trace!("palette: {:?}", palette);
        trace!("blocks: {:?}", blocks);

        let data = {
            let data = match palette {
                Palette::Direct | Palette::Indirect(..) => {
                    let blocks_per_long = 64 / bit_length;
                    let mut data = vec![0i64; (16 * 16 * 16) / blocks_per_long as usize];
                    let mut blocks_so_far = 0;
                    let mut long_index = 0;

                    for block in blocks {
                        if blocks_so_far == blocks_per_long {
                            blocks_so_far = 0;
                            long_index += 1;
                        }

                        let block = block as i64;

                        data[long_index] |= block << (blocks_so_far * bit_length);
                        blocks_so_far += 1;
                        trace!("block: {} ({:b}), long (after appending): {:b}, blocks so far: {blocks_so_far}", block, block, data[long_index])
                    }

                    data
                }
                Palette::SingleValued(_) => Vec::with_capacity(0),
            };

            let data = fastnbt::LongArray::new(data.to_vec());
            trace!("data: {:?}", data);
            data
        };

        Self {
            block_count: block_count as i16,
            block_states: PalettedContainer {
                bits_per_entry: bit_length,
                palette,
                data_array: data,
            },
            biomes: PalettedContainer {
                bits_per_entry: 0,
                palette: Palette::SingleValued(BlockState(0)),
                data_array: fastnbt::LongArray::new(vec![]),
            },
        }
    }
}

impl From<&world::Chunk> for ChunkDataUpdateLightC<'_> {
    fn from(value: &world::Chunk) -> Self {
        let data = value
            .sections
            .iter()
            .map(ChunkSection::anvil_to_sec)
            .collect::<Vec<_>>();

        Self {
            x: value.x_pos,
            z: value.z_pos,
            heightmaps: HeightMaps(HashMap::new()),
            data,
            entities: vec![],
            sky_light_mask: BitVec::from_elem(18, false),
            block_light_mask: BitVec::from_elem(18, false),
            empty_sky_light_mask: BitVec::from_elem(18, true),
            empty_block_light_mask: BitVec::from_elem(18, true),
            sky_light_arrays: vec![],
            block_light_arrays: vec![],
        }
    }
}