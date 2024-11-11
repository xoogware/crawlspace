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

use color_eyre::eyre::Result;
use serde::Deserialize;

use super::Block;

pub static ALL_BLOCKS: LazyLock<Blocks> = LazyLock::new(|| {
    serde_json::from_str(include_str!("../../assets/blocks.json"))
        .expect("blocks.json should be parseable")
});

#[derive(Debug, Deserialize)]
pub struct Blocks(HashMap<String, PossibleBlock>);

#[derive(Debug, Clone, Copy)]
pub struct BlockState(pub u16);

impl BlockState {
    pub const AIR: Self = Self(0);
}

#[derive(Debug, thiserror::Error)]
pub enum BlockStateError {
    #[error("Block not found")]
    NotFound,
}

impl TryFrom<&Block> for BlockState {
    type Error = BlockStateError;

    fn try_from(value: &Block) -> Result<Self, Self::Error> {
        // TODO: build map lazily to speed up load time?
        ALL_BLOCKS
            .0
            .get(&value.name)
            .and_then(|b| {
                b.states
                    .iter()
                    .find(|s| s.properties == value.properties)
                    .map(|b| Self(b.id))
            })
            .ok_or(BlockStateError::NotFound)
    }
}

#[derive(Debug, Deserialize)]
struct PossibleBlock {
    states: Vec<PossibleBlockState>,
}

#[derive(Debug, Deserialize)]
struct PossibleBlockState {
    id: u16,
    #[serde(default)]
    properties: HashMap<String, String>,
}
