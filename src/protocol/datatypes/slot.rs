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

use crate::{protocol::Encode, server::registries::REGISTRIES, world::Item};

use super::VarInt;

#[derive(Debug, Clone)]
pub struct Slot {
    item_count: i8,
    item_id: Option<i32>,
    components_to_add: Option<Vec<Component>>,
    components_to_remove: Option<Vec<i32>>,
}

#[derive(Debug, Clone)]
pub enum Component {}

impl From<Item> for Slot {
    fn from(value: Item) -> Self {
        let item_id = REGISTRIES
            .item
            .entries
            .get(&value.id)
            .expect("Couldn't find registry entry for item")
            .protocol_id;

        debug!("item id for {}: {item_id}", value.id);

        Self {
            item_count: value.count as i8,
            item_id: Some(item_id),
            components_to_add: None,
            components_to_remove: None,
        }
    }
}

impl Default for Slot {
    fn default() -> Self {
        Self {
            item_count: 0,
            item_id: None,
            components_to_add: None,
            components_to_remove: None,
        }
    }
}

impl Encode for Slot {
    fn encode(&self, mut w: impl std::io::Write) -> color_eyre::eyre::Result<()> {
        self.item_count.encode(&mut w)?;

        if self.item_count == 0 {
            return Ok(());
        }

        if let Some(item_id) = self.item_id {
            VarInt(item_id).encode(&mut w)?;

            VarInt(
                self.components_to_add
                    .as_ref()
                    .map(|v| v.len())
                    .unwrap_or(0) as i32,
            )
            .encode(&mut w)?;

            VarInt(
                self.components_to_remove
                    .as_ref()
                    .map(|v| v.len())
                    .unwrap_or(0) as i32,
            )
            .encode(&mut w)?;

            if let Some(ref components_to_add) = self.components_to_add {
                for _component in components_to_add {
                    unimplemented!("Encoding components is not implemented");
                }
            }

            if let Some(ref components_to_remove) = self.components_to_remove {
                for _component in components_to_remove {
                    unimplemented!("Encoding components is not implemented");
                }
            }
        }

        Ok(())
    }
}
