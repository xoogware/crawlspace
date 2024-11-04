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

use serde::{Deserialize, Serialize};

use super::{RegistryItem, StringOrCompound};

#[derive(Debug, Serialize, Deserialize)]
pub struct TrimMaterial {
    asset_name: String,
    ingredient: String,
    item_model_index: f32,
    description: StringOrCompound<TrimDescription>,
}

#[derive(Debug, Serialize, Deserialize)]
struct TrimDescription {
    color: String,
    translate: String,
}

impl RegistryItem for TrimMaterial {
    const ID: &str = "minecraft:trim_material";
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TrimPattern {
    asset_id: String,
    template_item: String,
    description: TrimDescription,
    decal: i8,
}

impl RegistryItem for TrimPattern {
    const ID: &str = "minecraft:trim_pattern";
}
