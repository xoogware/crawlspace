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

use crate::protocol::{Decode, Encode, Packet};

#[derive(Debug)]
pub struct KeepAliveC(pub i64);

impl Packet for KeepAliveC {
    const ID: i32 = 0x26;
}

impl Encode for KeepAliveC {
    fn encode(&self, mut w: impl std::io::Write) -> color_eyre::eyre::Result<()> {
        self.0.encode(&mut w)
    }
}

#[derive(Debug)]
#[expect(unused)]
pub struct KeepAliveS(i64);

impl Packet for KeepAliveS {
    const ID: i32 = 0x18;
}

impl<'a> Decode<'a> for KeepAliveS {
    fn decode(r: &mut &'a [u8]) -> color_eyre::eyre::Result<Self> {
        Ok(Self(i64::decode(r)?))
    }
}
