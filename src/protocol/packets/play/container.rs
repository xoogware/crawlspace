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
    protocol::{
        datatypes::{TextComponent, VarInt},
        Encode, Packet,
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
