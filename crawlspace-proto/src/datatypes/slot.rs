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

// use crate::{protocol::Encode, server::registries::REGISTRIES, world::Item};
use crate::{ErrorKind, Write};

use super::VarInt;

#[derive(Debug, Clone, Default)]
pub struct Slot {
    item_count: i8,
    item_id: Option<i32>,
    components_to_add: Option<Vec<Component>>,
    components_to_remove: Option<Vec<i32>>,
}

#[derive(Debug, Clone)]
pub enum Component {}

impl Slot {
    pub fn new(item_id: i32, item_count: i8) -> Self {
        Self {
            item_count,
            item_id: Some(item_id),
            components_to_add: None,
            components_to_remove: None,
        }
    }
}

impl Write for Slot {
    fn write(&self, w: &mut impl std::io::Write) -> Result<(), ErrorKind> {
        self.item_count.write(w)?;

        if self.item_count == 0 {
            return Ok(());
        }

        if let Some(item_id) = self.item_id {
            VarInt(item_id).write(w)?;

            VarInt(
                self.components_to_add
                    .as_ref()
                    .map(|v| v.len())
                    .unwrap_or(0) as i32,
            )
            .write(w)?;

            VarInt(
                self.components_to_remove
                    .as_ref()
                    .map(|v| v.len())
                    .unwrap_or(0) as i32,
            )
            .write(w)?;

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
