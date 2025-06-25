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

use crate::protocol::datatypes::Slot;

use super::BlockEntity;

#[derive(Debug, Clone)]
pub struct Container(pub Vec<Slot>);

#[derive(Debug, thiserror::Error)]
pub enum ContainerCreationError {
    #[error("Block entity is not a container")]
    NotAContainer,
    #[error("Parse error: {0}")]
    ParseError(color_eyre::eyre::Report),
}

impl TryFrom<BlockEntity> for Container {
    type Error = ContainerCreationError;

    fn try_from(value: BlockEntity) -> Result<Self, Self::Error> {
        match value.id.as_str() {
            "minecraft:chest" | "minecraft:trapped_chest" | "minecraft:barrel" => {
                let items = value
                    .try_get_items()
                    .map_err(ContainerCreationError::ParseError)?;

                let mut slots = vec![Slot::default(); 27];

                for item in items {
                    let slot_index = item.slot as usize;
                    slots[slot_index] = Slot::from(item);
                }

                Ok(Self(slots))
            }
            _ => Err(ContainerCreationError::NotAContainer),
        }
    }
}
