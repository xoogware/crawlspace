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

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct TextComponent {
    text: String,
}

impl From<String> for TextComponent {
    fn from(value: String) -> Self {
        Self { text: value }
    }
}
impl From<&str> for TextComponent {
    fn from(value: &str) -> Self {
        Self {
            text: value.to_owned(),
        }
    }
}
