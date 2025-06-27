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

use serde::Deserialize;

use super::Block;

#[derive(Debug, Deserialize)]
pub struct Blocks(HashMap<String, PossibleBlock>);

impl Blocks {
    pub fn new() -> Self {
        serde_json::from_str(include_str!("../../assets/blocks.json"))
            .expect("blocks.json should be parseable")
    }
}

#[derive(Debug, Clone, Copy)]
pub struct BlockState(pub u16);

impl BlockState {
    pub const AIR: Self = Self(0);
}

impl BlockState {
    pub fn parse_state(value: &Block, block_states: &Blocks) -> Option<Self> {
        // TODO: build map lazily to speed up load time?
        block_states.0.get(&value.name).and_then(|b| {
            b.states
                .iter()
                .find(|s| s.properties == value.properties)
                .map(|b| Self(b.id))
        })
    }
}

#[derive(Debug, Deserialize, Clone)]
struct PossibleBlock {
    states: Vec<PossibleBlockState>,
}

#[derive(Debug, Deserialize, Clone)]
struct PossibleBlockState {
    id: u16,
    #[serde(default)]
    properties: HashMap<String, String>,
}
