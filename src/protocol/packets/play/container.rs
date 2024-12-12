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

use byteorder::{BigEndian, ReadBytesExt};
use bytes::Buf;

use crate::{
    protocol::{
        datatypes::{Slot, TextComponent, VarInt},
        Decode, Encode, Packet,
    },
    server::window::{Window, WindowType},
};

#[derive(Debug)]
pub struct OpenScreenC {
    window_id: i32,
    window_type: WindowType,
    window_title: TextComponent,
}

impl Packet for OpenScreenC {
    const ID: i32 = 0x33;
}

impl Encode for OpenScreenC {
    fn encode(&self, mut w: impl std::io::Write) -> color_eyre::eyre::Result<()> {
        VarInt(self.window_id).encode(&mut w)?;
        VarInt(self.window_type as i32).encode(&mut w)?;
        fastnbt::to_bytes_with_opts(&self.window_title, fastnbt::SerOpts::network_nbt())?
            .encode(&mut w)?;

        Ok(())
    }
}

impl From<&Window> for OpenScreenC {
    fn from(value: &Window) -> Self {
        Self {
            window_id: value.id as i32,
            window_type: value.kind,
            window_title: value.title.clone(),
        }
    }
}

#[derive(Debug)]
pub struct SetContainerContentC {
    pub window_id: u8,
    pub state_id: i32,
    pub slot_data: Vec<Slot>,
    pub carried_item: Slot,
}

impl Packet for SetContainerContentC {
    const ID: i32 = 0x13;
}

impl Encode for SetContainerContentC {
    fn encode(&self, mut w: impl std::io::Write) -> color_eyre::eyre::Result<()> {
        self.window_id.encode(&mut w)?;
        VarInt(self.state_id).encode(&mut w)?;
        VarInt(self.slot_data.len() as i32).encode(&mut w)?;

        for slot in &self.slot_data {
            slot.encode(&mut w)?;
        }

        self.carried_item.encode(&mut w)?;

        Ok(())
    }
}

#[derive(Debug)]
pub struct ClickContainerS {
    pub window_id: u8,
    pub state_id: i32,
    pub slot: i16,
    pub button: i8,
    pub mode: InventoryMode,
    pub changed_slots: Vec<(i16, Slot)>,
    pub carried_item: Slot,
}

#[derive(Debug)]
#[repr(i32)]
pub enum InventoryMode {
    Mode0,
    Mode1,
    Mode2,
    Mode3,
    Mode4,
    Mode5,
    Mode6,
    Invalid,
}

impl From<i32> for InventoryMode {
    fn from(value: i32) -> Self {
        match value {
            0 => Self::Mode0,
            1 => Self::Mode1,
            2 => Self::Mode2,
            3 => Self::Mode3,
            4 => Self::Mode4,
            5 => Self::Mode5,
            6 => Self::Mode6,
            _ => Self::Invalid,
        }
    }
}

impl Packet for ClickContainerS {
    const ID: i32 = 0x0E;
}

impl Decode<'_> for ClickContainerS {
    fn decode(r: &mut &'_ [u8]) -> color_eyre::eyre::Result<Self>
    where
        Self: Sized,
    {
        let window_id = r.read_u8()?;
        let state_id = VarInt::decode(r)?.0;
        let slot = r.read_i16::<BigEndian>()?;
        let button = r.read_i8()?;
        let mode = r.read_i32::<BigEndian>()?.into();

        let changed_slot_len = VarInt::decode(r)?.0;

        let mut changed_slots = Vec::new();
        for _ in 0..changed_slot_len {
            changed_slots.push((r.read_i16::<BigEndian>()?, Slot::decode(r)?));
        }

        let carried_item = Slot::decode(r)?;

        Ok(Self {
            window_id,
            state_id,
            slot,
            button,
            mode,
            changed_slots,
            carried_item,
        })
    }
}
