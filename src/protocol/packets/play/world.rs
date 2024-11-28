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

use std::{collections::HashMap, sync::Arc};

use bit_vec::BitVec;
use bytes::BufMut;
use fastnbt::SerOpts;

use crate::{
    protocol::{
        datatypes::{VarInt, VarLong},
        Encode, Packet,
    },
    world::{
        self,
        blocks::{BlockState, Blocks},
    },
    CrawlState,
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
    entities: Vec<BlockEntity>,
    sky_light_mask: BitVec,
    block_light_mask: BitVec,
    empty_sky_light_mask: BitVec,
    empty_block_light_mask: BitVec,
    sky_light_arrays: Vec<&'a [u8]>,
    block_light_arrays: Vec<&'a [u8]>,
}

#[derive(Debug)]
struct BlockEntity {
    packed_xz: u8,
    y: i16,
    kind: VarInt,
    data: Vec<u8>,
}

impl Encode for BlockEntity {
    fn encode(&self, mut w: impl std::io::Write) -> color_eyre::eyre::Result<()> {
        self.packed_xz.encode(&mut w)?;
        self.y.encode(&mut w)?;
        self.kind.encode(&mut w)?;
        self.data.encode(&mut w)?;

        Ok(())
    }
}

impl From<world::BlockEntity> for BlockEntity {
    fn from(value: world::BlockEntity) -> Self {
        let data = fastnbt::to_bytes_with_opts(&value.raw_data, fastnbt::SerOpts::network_nbt())
            .expect("Failed to parse network nbt for block entity");

        let kind = VarInt(match value.id.as_str() {
            "minecraft:furnace" => 0,
            "minecraft:chest" => 1,
            "minecraft:trapped_chest" => 2,
            "minecraft:ender_chest" => 3,
            "minecraft:jukebox" => 4,
            "minecraft:dispenser" => 5,
            "minecraft:dropper" => 6,
            "minecraft:sign" => 7,
            "minecraft:hanging_sign" => 8,
            "minecraft:mob_spawner" => 9,
            "minecraft:piston" => 10,
            "minecraft:brewing_stand" => 11,
            "minecraft:enchanting_table" => 12,
            "minecraft:end_portal" => 13,
            "minecraft:beacon" => 14,
            "minecraft:skull" => 15,
            "minecraft:daylight_detector" => 16,
            "minecraft:hopper" => 17,
            "minecraft:comparator" => 18,
            "minecraft:banner" => 19,
            "minecraft:structure_block" => 20,
            "minecraft:end_gateway" => 21,
            "minecraft:command_block" => 22,
            "minecraft:shulker_box" => 23,
            "minecraft:bed" => 24,
            "minecraft:conduit" => 25,
            "minecraft:barrel" => 26,
            "minecraft:smoker" => 27,
            "minecraft:blast_furnace" => 28,
            "minecraft:lectern" => 29,
            "minecraft:bell" => 30,
            "minecraft:jigsaw" => 31,
            "minecraft:campfire" => 32,
            "minecraft:beehive" => 33,
            "minecraft:sculk_sensor" => 34,
            "minecraft:calibrated_sculk_sensor" => 35,
            "minecraft:sculk_catalyst" => 36,
            "minecraft:sculk_shrieker" => 37,
            "minecraft:chiseled_bookshelf" => 38,
            "minecraft:brushable_block" => 39,
            "minecraft:decorated_pot" => 40,
            "minecraft:crafter" => 41,
            "minecraft:trial_spawner" => 42,
            "minecraft:vault" => 43,
            i => unimplemented!("Block Entity type {i} is unimplemented"),
        });

        Self {
            packed_xz: (((value.x & 15) << 4) | (value.z & 15)) as u8,
            y: value.y as i16,
            kind,
            data,
        }
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
    pub fn anvil_to_sec(
        crawlstate: CrawlState,
        value: &world::Section,
        block_states: &Blocks,
    ) -> Self {
        let mut blocks = Vec::new();
        let bit_length = (64 - (value.block_states.palette.len() as u64).leading_zeros()).max(4);

        let blocks_per_long = 64 / bit_length;

        #[cfg(not(feature = "modern_art"))]
        let bit_mask = (1 << bit_length) - 1;

        match value.block_states.data {
            None => blocks.fill(0),
            Some(ref data) => {
                trace!("data.len(): {}", data.len());
                trace!("blocks_per_long: {blocks_per_long}");
                blocks.resize(data.len() * blocks_per_long as usize, 0);
                let mut i = 0;
                for long in data.iter() {
                    #[cfg(not(feature = "modern_art"))]
                    {
                        let long = *long as u64;
                        for b in 0..blocks_per_long {
                            blocks[i] = ((long >> (bit_length * b)) & bit_mask) as i16;
                            i += 1;
                        }
                    }

                    #[cfg(feature = "modern_art")]
                    {
                        let mut long = *long as u64;
                        while long != 0 {
                            blocks[i] = (long & ((1 << bit_length) - 1)) as i16;
                            long >>= bit_length;
                            i += 1;
                        }
                    }
                }
            }
        }

        let palette = value
            .block_states
            .palette
            .iter()
            .map(|b| BlockState::parse_state(b, block_states).unwrap_or(BlockState::AIR))
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

        trace!("bit_length: {bit_length}");

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
                    let mut data = vec![0i64; blocks.len() / blocks_per_long as usize];
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
                palette: Palette::SingleValued(BlockState(
                    crawlstate.registry_cache.the_end_biome_id,
                )),
                data_array: fastnbt::LongArray::new(vec![]),
            },
        }
    }
}

impl ChunkDataUpdateLightC<'_> {
    pub fn new(crawlstate: CrawlState, value: &world::Chunk, block_states: &Blocks) -> Self {
        let data = value
            .sections
            .iter()
            .map(|sec| ChunkSection::anvil_to_sec(crawlstate.clone(), sec, block_states))
            .collect::<Vec<_>>();

        let block_entities = value
            .block_entities
            .clone()
            .into_iter()
            .filter_map(|e| {
                world::BlockEntity::try_parse(e).map_or_else(
                    |why| {
                        warn!(
                            "Failed to parse block entity: {why}, ignoring in final chunk packet for ({}, {})",
                            value.x_pos,
                            value.z_pos,
                        );
                        None
                    },
                    |e| match e.keep_packed {
                        true => None,
                        false => Some(e)
                    },
                )
            })
            .map(Into::into)
            .collect::<Vec<self::BlockEntity>>();

        Self {
            x: value.x_pos,
            z: value.z_pos,
            heightmaps: HeightMaps(HashMap::new()),
            data,
            entities: block_entities,
            sky_light_mask: BitVec::from_elem(18, false),
            block_light_mask: BitVec::from_elem(18, false),
            empty_sky_light_mask: BitVec::from_elem(18, true),
            empty_block_light_mask: BitVec::from_elem(18, true),
            sky_light_arrays: vec![],
            block_light_arrays: vec![],
        }
    }
}

#[derive(Debug)]
pub struct InitializeWorldBorderC {
    pub x: f64,
    pub z: f64,
    pub old_diameter: f64,
    pub new_diameter: f64,
    pub speed: i64,
    pub teleport_boundary: i32,
    pub warning_blocks: i32,
    pub warning_time_sec: i32,
}

impl Packet for InitializeWorldBorderC {
    const ID: i32 = 0x25;
}

impl Encode for InitializeWorldBorderC {
    fn encode(&self, mut w: impl std::io::Write) -> color_eyre::eyre::Result<()> {
        self.x.encode(&mut w)?;
        self.z.encode(&mut w)?;
        self.old_diameter.encode(&mut w)?;
        self.new_diameter.encode(&mut w)?;
        VarLong(self.speed).encode(&mut w)?;
        VarInt(self.teleport_boundary).encode(&mut w)?;
        VarInt(self.warning_blocks).encode(&mut w)?;
        VarInt(self.warning_time_sec).encode(&mut w)?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct SetBorderCenterC {
    pub x: f64,
    pub z: f64,
}

impl Packet for SetBorderCenterC {
    const ID: i32 = 0x4D;
}

impl Encode for SetBorderCenterC {
    fn encode(&self, mut w: impl std::io::Write) -> color_eyre::eyre::Result<()> {
        self.x.encode(&mut w)?;
        self.z.encode(&mut w)?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct SetBorderSizeC(pub f64);

impl Packet for SetBorderSizeC {
    const ID: i32 = 0x4F;
}

impl Encode for SetBorderSizeC {
    fn encode(&self, w: impl std::io::Write) -> color_eyre::eyre::Result<()> {
        self.0.encode(w)
    }
}
