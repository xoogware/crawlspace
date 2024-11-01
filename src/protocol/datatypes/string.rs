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

use color_eyre::eyre::{ensure, Result};

use crate::protocol::{Decode, Encode};

use super::VarInt;

#[derive(Debug)]
pub struct BoundedString<'a, const BOUND: usize = 32767>(pub &'a str);

impl<'a, const BOUND: usize> Decode<'a> for BoundedString<'a, BOUND> {
    fn decode(r: &mut &'a [u8]) -> Result<Self> {
        let len = VarInt::decode(r)?.0;
        ensure!(len >= 0, "tried to decode string with negative length");

        let len = len as usize;
        ensure!(
            len <= r.len(),
            "malformed packet - not enough data to continue decoding (expected {len} got {})",
            r.len(),
        );

        let (content, rest) = r.split_at(len);
        let content = std::str::from_utf8(content)?;
        let utf16_len = content.encode_utf16().count();

        ensure!(
            utf16_len <= BOUND,
            "utf-16 encoded string exceeds {BOUND} chars (is {utf16_len})"
        );

        *r = rest;

        Ok(BoundedString(content))
    }
}

impl<'a, const BOUND: usize> Encode for BoundedString<'a, BOUND> {
    fn encode(&self, mut w: impl std::io::Write) -> Result<()> {
        let len = self.0.encode_utf16().count();

        ensure!(len < BOUND, "length of string {len} exceeds bound {BOUND}");

        VarInt(self.0.len() as i32).encode(&mut w)?;
        Ok(w.write_all(self.0.as_bytes())?)
    }
}

impl Encode for str {
    fn encode(&self, mut w: impl std::io::Write) -> Result<()> {
        VarInt(self.len() as i32).encode(&mut w)?;
        Ok(w.write_all(self.as_bytes())?)
    }
}
