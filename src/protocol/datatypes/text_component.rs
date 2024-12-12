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

use fastnbt::{DeOpts, SerOpts};
use serde::{Deserialize, Serialize};

use crate::protocol::{Decode, Encode};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TextComponent {
    String { text: String },
    Compound,
}

impl From<String> for TextComponent {
    fn from(value: String) -> Self {
        Self::String { text: value }
    }
}
impl From<&str> for TextComponent {
    fn from(value: &str) -> Self {
        Self::String {
            text: value.to_owned(),
        }
    }
}

impl Encode for TextComponent {
    fn encode(&self, mut w: impl std::io::Write) -> color_eyre::eyre::Result<()> {
        fastnbt::to_bytes_with_opts(self, SerOpts::network_nbt())?.encode(&mut w)?;

        Ok(())
    }
}

impl Decode<'_> for TextComponent {
    fn decode(r: &mut &'_ [u8]) -> color_eyre::eyre::Result<Self>
    where
        Self: Sized,
    {
        match fastnbt::from_bytes_with_opts::<String>(r, DeOpts::network_nbt()) {
            Ok(s) => Ok(TextComponent::from(s)),
            Err(_) => Ok(fastnbt::from_bytes_with_opts(r, DeOpts::network_nbt())?),
        }
    }
}
