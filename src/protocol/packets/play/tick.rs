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

use crate::protocol::{datatypes::VarInt, Encode, Packet};

#[derive(Debug)]
pub struct SetTickingStateC {
    pub tick_rate: f32,
    pub is_frozen: bool,
}

impl Packet for SetTickingStateC {
    const ID: i32 = 0x71;
}

impl Encode for SetTickingStateC {
    fn encode(&self, mut w: impl std::io::Write) -> color_eyre::eyre::Result<()> {
        self.tick_rate.encode(&mut w)?;
        self.is_frozen.encode(&mut w)?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct StepTicksC(pub i32);

impl Packet for StepTicksC {
    const ID: i32 = 0x72;
}

impl Encode for StepTicksC {
    fn encode(&self, w: impl std::io::Write) -> color_eyre::eyre::Result<()> {
        VarInt(self.0).encode(w)
    }
}
