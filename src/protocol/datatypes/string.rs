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

use super::{Bytes, VarInt};

#[derive(Debug)]
pub struct Bounded<T, const BOUND: usize = 32767>(pub T);

impl<'a, const BOUND: usize> Decode<'a> for Bounded<&'a str, BOUND> {
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

        Ok(Bounded(content))
    }
}

impl<'a, const BOUND: usize> Encode for Bounded<&'a str, BOUND> {
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

impl<'a, const BOUND: usize> Encode for Bounded<Bytes<'a>, BOUND> {
    fn encode(&self, mut w: impl std::io::Write) -> Result<()> {
        let len = self.0 .0.len();
        ensure!(len < BOUND, "length of bytes {len} exceeds bound {BOUND}");
        VarInt(len as i32).encode(&mut w)?;
        Ok(self.0.encode(&mut w)?)
    }
}

impl<'a, const BOUND: usize> Decode<'a> for Bounded<Bytes<'a>, BOUND> {
    fn decode(r: &mut &'a [u8]) -> Result<Self> {
        let len = VarInt::decode(r)?.0;
        ensure!(len >= 0, "tried to decode string with negative length");

        let len = len as usize;
        ensure!(
            len <= r.len(),
            "malformed packet - not enough data to continue decoding (expected {len} got {})",
            r.len(),
        );

        let (mut content, rest) = r.split_at(len);
        let content = Bytes::decode(&mut content)?;
        let len = content.0.len();

        ensure!(
            len <= BOUND,
            "raw byte length exceeds {BOUND} chars (is {len})"
        );

        *r = rest;

        Ok(Bounded(content))
    }
}
