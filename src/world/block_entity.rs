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

use color_eyre::eyre::{bail, Result};
use fastnbt::Value;
use serde::Deserialize;
use serde_with::{serde_as, EnumMap};

macro_rules! get_tag {
    ($data:expr, $tag_kind:path, $tag_name:literal) => {{
        {
            let tag = $data.get($tag_name);

            let Some(tag) = tag else {
                bail!("Unable to find tag {}: {:?}", $tag_name, $data);
            };

            match tag {
                $tag_kind(v) => v.to_owned(),
                e => bail!(
                    "Improper tag kind found searching for {}: {:?}",
                    $tag_name,
                    e
                ),
            }
        }
    }};
}

pub struct BlockEntity {
    pub id: String,
    pub keep_packed: bool,
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub raw_data: fastnbt::Value,
}

impl BlockEntity {
    pub fn try_parse(value: Value) -> Result<Self> {
        let raw_data = value.clone();

        let Value::Compound(data) = value else {
            bail!(
                "try_parse was called with a value that is not a compound: {:?}",
                value
            );
        };

        let id = get_tag!(data, Value::String, "id");
        trace!("Parsing {id}");

        Ok(Self {
            id,
            keep_packed: get_tag!(data, Value::Byte, "keepPacked") == 1,
            x: get_tag!(data, Value::Int, "x"),
            y: get_tag!(data, Value::Int, "y"),
            z: get_tag!(data, Value::Int, "z"),
            raw_data,
        })
    }

    pub fn try_get_items(&self) -> Result<Vec<Item>> {
        match self.id.as_str() {
            "minecraft:chest" | "minecraft:trapped_chest" | "minecraft:barrel" => {
                let Value::Compound(ref data) = self.raw_data else {
                    bail!(
                        "try_get_items was called with raw_data that is not a compound: {:?}",
                        self.raw_data
                    );
                };

                let items = get_tag!(data, Value::List, "Items");
                Ok(items
                    .iter()
                    .map(|i| fastnbt::from_value::<Item>(i).expect("Failed to parse item"))
                    .collect())
            }
            id => bail!("try_get_items called on not a container ({id})"),
        }
    }
}

#[serde_as]
#[derive(Debug, Clone, Deserialize)]
pub struct Item {
    #[serde(rename = "Slot")]
    pub slot: i8,
    pub id: String,
    pub count: i32,
    #[serde_as(as = "EnumMap")]
    pub components: Vec<Component>,
}

#[derive(Debug, Clone, Deserialize)]
pub enum Component {
    #[serde(rename = "minecraft:written_book_content")]
    WrittenBookContent {
        pages: Vec<Page>,
        title: Text,
        author: String,
        #[serde(default)]
        generation: Generation,
        #[serde(default)]
        resolved: bool,
    },
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum Page {
    Text(Text),
    String(String),
}

#[derive(Debug, Clone, Deserialize)]
pub struct Text {
    pub raw: String,
    pub filtered: Option<String>,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[repr(i32)]
pub enum Generation {
    Original,
    CopyOfOriginal,
    CopyOfCopy,
    Tattered,
}

#[derive(Debug, thiserror::Error)]
pub enum GenerationDecodeError {
    #[error("Invalid tag: {0}")]
    Invalid(i32),
}

impl Default for Generation {
    fn default() -> Self {
        Self::Original
    }
}
