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

use crate::{
    protocol::{Decode, Encode},
    server::registries::REGISTRIES,
    world::{self, Item},
};

use super::{TextComponent, VarInt};

#[derive(Debug, Clone)]
pub struct Slot {
    item_count: i8,
    item_id: i32,
    components_to_add: Vec<Component>,
    components_to_remove: Vec<i32>,
}

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
            item_id,
            components_to_add: value.components.iter().map(Component::from).collect(),
            components_to_remove: Vec::new(),
        }
    }
}

impl Default for Slot {
    fn default() -> Self {
        // FIXME: probably use an enum for empty slots to avoid allocating vecs
        Self {
            item_count: 0,
            item_id: 0,
            components_to_add: Vec::new(),
            components_to_remove: Vec::new(),
        }
    }
}

impl Encode for Slot {
    fn encode(&self, mut w: impl std::io::Write) -> color_eyre::eyre::Result<()> {
        self.item_count.encode(&mut w)?;

        if self.item_count == 0 {
            return Ok(());
        }

        VarInt(self.item_id).encode(&mut w)?;
        VarInt(self.components_to_add.len() as i32).encode(&mut w)?;
        VarInt(self.components_to_remove.len() as i32).encode(&mut w)?;

        for component in &self.components_to_add {
            component.encode(&mut w)?;
        }

        for component in &self.components_to_remove {
            component.encode(&mut w)?;
        }

        Ok(())
    }
}

impl Decode<'_> for Slot {
    fn decode(r: &mut &'_ [u8]) -> color_eyre::eyre::Result<Self>
    where
        Self: Sized,
    {
        let item_count = VarInt::decode(r)?.0 as i8;

        let (item_id, components_to_add, components_to_remove) = match item_count {
            0 => (0, Vec::new(), Vec::new()),
            _ => {
                let item_id = VarInt::decode(r)?.0;
                let number_components_to_add = VarInt::decode(r)?.0;
                let number_components_to_remove = VarInt::decode(r)?.0;

                let mut components_to_add = Vec::new();
                let mut components_to_remove = Vec::new();

                for _ in 0..number_components_to_add {
                    components_to_add.push(Component::decode(r)?);
                }

                for _ in 0..number_components_to_remove {
                    components_to_remove.push(VarInt::decode(r)?.0);
                }

                (item_id, components_to_add, components_to_remove)
            }
        };

        Ok(Self {
            item_count,
            item_id,
            components_to_add,
            components_to_remove,
        })
    }
}

#[derive(Debug, Clone)]
pub enum Component {
    WrittenBookContent {
        raw_title: String,
        filtered_title: Option<String>,
        author: String,
        generation: VarInt,
        pages: Vec<Page>,
        resolved: bool,
    },
    Unknown(i32),
}

#[derive(Debug, Clone)]
pub struct Page {
    raw_content: TextComponent,
    filtered_content: Option<TextComponent>,
}

impl Decode<'_> for Page {
    fn decode(r: &mut &'_ [u8]) -> color_eyre::eyre::Result<Self>
    where
        Self: Sized,
    {
        let raw_content = TextComponent::decode(r)?;
        let has_filtered_content = bool::decode(r)?;
        let filtered_content = match has_filtered_content {
            true => Some(TextComponent::decode(r)?),
            false => None,
        };

        Ok(Self {
            raw_content,
            filtered_content,
        })
    }
}

impl From<&world::Component> for Component {
    fn from(value: &world::Component) -> Self {
        match value {
            world::Component::WrittenBookContent {
                pages,
                title,
                author,
                generation,
                resolved,
            } => Self::WrittenBookContent {
                raw_title: title.raw.to_owned(),
                filtered_title: title.filtered.to_owned(),
                author: author.to_owned(),
                generation: VarInt(*generation as i32),
                pages: pages.iter().map(Page::from).collect(),
                resolved: *resolved,
            },
        }
    }
}

impl From<&world::Page> for Page {
    fn from(value: &world::Page) -> Self {
        match value {
            world::Page::Text(text) => Self {
                raw_content: TextComponent::from(text.raw.to_owned()),
                filtered_content: text.filtered.to_owned().map(TextComponent::from),
            },
            world::Page::String(s) => Self {
                raw_content: TextComponent::from(s.to_owned()),
                filtered_content: None,
            },
        }
    }
}

impl Component {
    fn id(&self) -> VarInt {
        VarInt(match self {
            Self::WrittenBookContent { .. } => 34,
            Self::Unknown(id) => panic!("id called on unknown component (id {})", id),
        })
    }
}

impl Encode for Component {
    fn encode(&self, mut w: impl std::io::Write) -> color_eyre::eyre::Result<()> {
        self.id().encode(&mut w)?;

        match self {
            Self::WrittenBookContent {
                raw_title,
                filtered_title,
                author,
                generation,
                pages,
                resolved,
            } => {
                raw_title.encode(&mut w)?;
                filtered_title.is_some().encode(&mut w)?;

                if let Some(filtered_title) = filtered_title {
                    filtered_title.encode(&mut w)?;
                }

                author.encode(&mut w)?;
                generation.encode(&mut w)?;

                VarInt(pages.len() as i32).encode(&mut w)?;

                for page in pages {
                    page.raw_content.encode(&mut w)?;
                    page.filtered_content.is_some().encode(&mut w)?;

                    if let Some(filtered_content) = &page.filtered_content {
                        filtered_content.encode(&mut w)?;
                    }
                }

                resolved.encode(&mut w)?;
            }
            Self::Unknown(_) => (),
        }

        Ok(())
    }
}

impl Decode<'_> for Component {
    fn decode(r: &mut &'_ [u8]) -> color_eyre::eyre::Result<Self>
    where
        Self: Sized,
    {
        let id = VarInt::decode(r)?.0;

        let component = match id {
            34 => {
                let raw_title = String::decode(r)?;

                let has_filtered_title = bool::decode(r)?;
                let filtered_title = match has_filtered_title {
                    true => Some(String::decode(r)?),
                    false => None,
                };

                let author = String::decode(r)?;
                let generation = VarInt::decode(r)?;

                let page_count = VarInt::decode(r)?.0;
                let mut pages = Vec::new();
                for _ in 0..page_count {
                    pages.push(Page::decode(r)?);
                }

                let resolved = bool::decode(r)?;

                Self::WrittenBookContent {
                    raw_title,
                    filtered_title,
                    author,
                    generation,
                    pages,
                    resolved,
                }
            }
            id => Self::Unknown(id),
        };

        Ok(component)
    }
}
