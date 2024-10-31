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

use crate::protocol::Decode;

use super::VarInt;

const MAX_BOUND: usize = 32767;

#[derive(Debug)]
pub struct BoundedString<'a, const BOUND: usize>(&'a str);

impl<'a, const BOUND: usize> Decode<'a> for BoundedString<'a, BOUND> {
    fn decode(r: &mut &'a [u8]) -> Result<Self> {
        let len = VarInt::decode(r)?.0;
        ensure!(len >= 0, "tried to decode string with negative length");

        let len = len as usize;
        ensure!(
            len <= MAX_BOUND,
            "string length greater than MAX_BOUND {MAX_BOUND}"
        );
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
