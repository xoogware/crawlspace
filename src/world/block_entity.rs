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

use color_eyre::eyre::{bail, Error, Result};
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

macro_rules! get_tag_opt {
    ($data:expr, $tag_kind:path, $tag_name:literal) => {{
        let tag = $data.get($tag_name);

        match tag {
            Some($tag_kind(v)) => Some(v.to_owned()),
            _ => None,
        }
    }};
}

pub struct BlockEntity {
    pub id: String,
    pub keep_packed: bool,
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub other_tags: Box<dyn BlockEntityTags>,
}

impl BlockEntity {
    pub fn try_parse(value: Value) -> Result<Self> {
        let Value::Compound(data) = value else {
            bail!(
                "try_parse was called with a value that is not a compound: {:?}",
                value
            );
        };

        let id = get_tag!(data, Value::String, "id");
        trace!("Parsing {id}");

        let other_tags: Box<dyn BlockEntityTags> = match id.as_str() {
            "minecraft:barrel" => Box::new(BarrelBlockEntityTags::try_parse(&data)?),
            "minecraft:brewing_stand" => Box::new(BrewingStandBlockEntityTags::try_parse(&data)?),
            "minecraft:campfire" => Box::new(CampfireBlockEntityTags::try_parse(&data)?),
            "minecraft:chiseled_bookshelf" => {
                Box::new(ChiseledBookshelfBlockEntityTags::try_parse(&data)?)
            }
            "minecraft:sign" => Box::new(SignBlockEntityTags::try_parse(&data)?),
            "minecraft:skull" => Box::new(SkullBlockEntityTags::try_parse(&data)?),
            t => unimplemented!("Got block with tag {t}, but it isn't implemented"),
        };

        Ok(Self {
            id,
            keep_packed: get_tag!(data, Value::String, "keepPacked") == "true",
            x: get_tag!(data, Value::Int, "x"),
            y: get_tag!(data, Value::Int, "y"),
            z: get_tag!(data, Value::Int, "z"),
            other_tags,
        })
    }
}

pub struct BlockEntityItem {
    pub slot: i8,
    pub id: String,
    pub count: i32,
}

pub trait BlockEntityTags {
    fn try_parse(from: &HashMap<String, Value>) -> Result<Self>
    where
        Self: Sized;
}

pub struct SignBlockEntityTags {
    pub is_waxed: bool,
    pub front_text: SignText,
    pub back_text: SignText,
}

pub struct SignText {
    pub has_glowing_text: bool,
    pub color: String,
    pub messages: Vec<String>,
}

fn byte_to_bool(byte: i8) -> bool {
    matches!(byte, 0)
}

impl BlockEntityTags for SignBlockEntityTags {
    fn try_parse(from: &HashMap<String, Value>) -> Result<Self> {
        let front_text_tag = get_tag!(from, Value::Compound, "front_text");
        let back_text_tag = get_tag!(from, Value::Compound, "back_text");

        let front_messages = get_tag!(front_text_tag, Value::List, "messages")
            .iter()
            .map(|v| match v {
                Value::String(s) => s.to_owned(),
                _ => "Unknown".to_owned(),
            })
            .collect();

        let back_messages = get_tag!(back_text_tag, Value::List, "messages")
            .iter()
            .map(|v| match v {
                Value::String(s) => s.to_owned(),
                _ => "Unknown".to_owned(),
            })
            .collect();

        Ok(Self {
            is_waxed: byte_to_bool(get_tag!(from, Value::Byte, "is_waxed")),
            front_text: SignText {
                has_glowing_text: byte_to_bool(get_tag!(
                    front_text_tag,
                    Value::Byte,
                    "has_glowing_text"
                )),
                color: get_tag!(front_text_tag, Value::String, "color").to_owned(),
                messages: front_messages,
            },
            back_text: SignText {
                has_glowing_text: byte_to_bool(get_tag!(
                    back_text_tag,
                    Value::Byte,
                    "has_glowing_text"
                )),
                color: get_tag!(back_text_tag, Value::String, "color").to_owned(),
                messages: back_messages,
            },
        })
    }
}

pub struct CampfireBlockEntityTags {
    cooking_times: Vec<i32>,
    cooking_total_times: Vec<i32>,
    items: Vec<BlockEntityItem>,
}

impl BlockEntityTags for CampfireBlockEntityTags {
    fn try_parse(from: &HashMap<String, Value>) -> Result<Self>
    where
        Self: Sized,
    {
        let cooking_times = get_tag!(from, Value::List, "CookingTimes")
            .iter()
            .map(|v| match v {
                Value::Int(s) => s.to_owned(),
                _ => 0,
            })
            .collect();

        let cooking_total_times = get_tag!(from, Value::List, "CookingTotalTimes")
            .iter()
            .map(|v| match v {
                Value::Int(s) => s.to_owned(),
                _ => 0,
            })
            .collect();

        let items = {
            let item_compounds = get_tag!(from, Value::List, "Items");
            let mut items = Vec::with_capacity(item_compounds.len());
            for c in item_compounds {
                match c {
                    Value::Compound(c) => items.push(BlockEntityItem {
                        slot: get_tag!(c, Value::Byte, "Slot"),
                        id: get_tag!(c, Value::String, "id"),
                        count: get_tag!(c, Value::Int, "count"),
                    }),
                    t => bail!("Wrong tag type parsing item: {:?}", t),
                }
            }
            items
        };

        Ok(Self {
            cooking_times,
            cooking_total_times,
            items,
        })
    }
}

pub struct ChiseledBookshelfBlockEntityTags {
    items: Vec<BlockEntityItem>,
    last_interacted_slot: i32,
}

impl BlockEntityTags for ChiseledBookshelfBlockEntityTags {
    fn try_parse(from: &HashMap<String, Value>) -> Result<Self>
    where
        Self: Sized,
    {
        let items = {
            let item_compounds = get_tag!(from, Value::List, "Items");
            let mut items = Vec::with_capacity(item_compounds.len());
            for c in item_compounds {
                match c {
                    Value::Compound(c) => items.push(BlockEntityItem {
                        slot: get_tag!(c, Value::Byte, "Slot"),
                        id: get_tag!(c, Value::String, "id"),
                        count: get_tag!(c, Value::Int, "count"),
                    }),
                    t => bail!("Wrong tag type parsing item: {:?}", t),
                }
            }
            items
        };

        Ok(Self {
            items,
            last_interacted_slot: get_tag!(from, Value::Int, "last_interacted_slot"),
        })
    }
}

pub struct BarrelBlockEntityTags {
    custom_name: Option<String>,
    items: Vec<BlockEntityItem>,
}

impl BlockEntityTags for BarrelBlockEntityTags {
    fn try_parse(from: &HashMap<String, Value>) -> Result<Self>
    where
        Self: Sized,
    {
        let items = {
            let item_compounds = get_tag!(from, Value::List, "Items");
            let mut items = Vec::with_capacity(item_compounds.len());
            for c in item_compounds {
                match c {
                    Value::Compound(c) => items.push(BlockEntityItem {
                        slot: get_tag!(c, Value::Byte, "Slot"),
                        id: get_tag!(c, Value::String, "id"),
                        count: get_tag!(c, Value::Int, "count"),
                    }),
                    t => bail!("Wrong tag type parsing item: {:?}", t),
                }
            }
            items
        };

        Ok(Self {
            custom_name: get_tag_opt!(from, Value::String, "CustomName"),
            items,
        })
    }
}

pub struct BrewingStandBlockEntityTags {
    brew_time: i16,
    custom_name: Option<String>,
    fuel: i8,
    items: Vec<BlockEntityItem>,
    lock: String,
}

impl BlockEntityTags for BrewingStandBlockEntityTags {
    fn try_parse(from: &HashMap<String, Value>) -> Result<Self>
    where
        Self: Sized,
    {
        let items = {
            let item_compounds = get_tag!(from, Value::List, "Items");
            let mut items = Vec::with_capacity(item_compounds.len());
            for c in item_compounds {
                match c {
                    Value::Compound(c) => items.push(BlockEntityItem {
                        slot: get_tag!(c, Value::Byte, "Slot"),
                        id: get_tag!(c, Value::String, "id"),
                        count: get_tag!(c, Value::Int, "count"),
                    }),
                    t => bail!("Wrong tag type parsing item: {:?}", t),
                }
            }
            items
        };

        Ok(Self {
            brew_time: get_tag!(from, Value::Short, "BrewTime"),
            custom_name: get_tag_opt!(from, Value::String, "CustomName"),
            fuel: get_tag!(from, Value::Byte, "Fuel"),
            items,
            lock: get_tag!(from, Value::String, "Lock"),
        })
    }
}

pub struct SkullBlockEntityTags {
    pub custom_name: Option<String>,
    pub note_block_sound: Option<String>,
    pub profile: SkullProfile,
}

pub enum SkullProfile {
    String(String),
    Compound {
        name: String,
        id: Vec<i32>,
        properties: Option<Vec<SkullProperty>>,
    },
}

pub struct SkullProperty {
    pub name: String,
    pub value: String,
    pub signature: Option<String>,
}

impl BlockEntityTags for SkullBlockEntityTags {
    fn try_parse(from: &HashMap<String, Value>) -> Result<Self>
    where
        Self: Sized,
    {
        let profile = match get_tag_opt!(from, Value::String, "profile") {
            Some(s) => SkullProfile::String(s),
            None => match get_tag_opt!(from, Value::Compound, "profile") {
                None => bail!("Profile not present for skull"),
                Some(c) => {
                    let id = get_tag!(from, Value::List, "id")
                        .iter()
                        .map(|v| match v {
                            Value::Int(s) => s.to_owned(),
                            _ => 0,
                        })
                        .collect();

                    let properties = {
                        match get_tag_opt!(from, Value::List, "properties") {
                            None => None,
                            Some(property_compounds) => {
                                let mut properties = Vec::with_capacity(property_compounds.len());
                                for c in property_compounds {
                                    match c {
                                        Value::Compound(s) => properties.push(SkullProperty {
                                            name: get_tag!(s, Value::String, "name"),
                                            value: get_tag!(s, Value::String, "value"),
                                            signature: get_tag_opt!(s, Value::String, "signature"),
                                        }),
                                        t => bail!("Wrong tag type parsing item: {:?}", t),
                                    }
                                }

                                Some(properties)
                            }
                        }
                    };

                    SkullProfile::Compound {
                        name: get_tag!(c, Value::String, "name"),
                        id,
                        properties,
                    }
                }
            },
        };

        Ok(Self {
            custom_name: get_tag_opt!(from, Value::String, "custom_name"),
            note_block_sound: get_tag_opt!(from, Value::String, "note_block_sound"),
            profile,
        })
    }
}
