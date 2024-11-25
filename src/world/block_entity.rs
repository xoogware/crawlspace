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

use color_eyre::eyre::{bail, Result};
use fastnbt::Value;

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
}
