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
    ErrorKind::{self, InvalidData},
    Read, Write,
};

use super::VarInt;

#[derive(Debug)]
pub struct Bounded<T, const BOUND: usize = 32767>(pub T);

impl<const BOUND: usize> Read<'_> for Bounded<String, BOUND> {
    fn read(r: &mut impl std::io::Read) -> Result<Self, ErrorKind> {
        let len = VarInt::read(r)?.0;
        if len < 0 {
            return Err(InvalidData(
                "tried to decode string with negative length".to_string(),
            ));
        }

        let len = len as usize;

        let mut buf = vec![0; len];
        r.read_exact(&mut buf)?;
        let content = String::from_utf8(buf)
            .map_err(|_| ErrorKind::InvalidData("invalid utf8 string data".to_string()))?;
        let utf16_len = content.encode_utf16().count();

        if utf16_len > BOUND {
            return Err(InvalidData(format!(
                "utf-16 encoded string exceeds {BOUND} chars (is {utf16_len})"
            )));
        }

        Ok(Bounded(content))
    }
}

impl<'a, const BOUND: usize> Write for Bounded<String, BOUND> {
    fn write(&self, w: &mut impl std::io::Write) -> Result<(), ErrorKind> {
        let len = self.0.encode_utf16().count();

        if len > BOUND {
            return Err(InvalidData(format!(
                "length of string {len} exceeds bound {BOUND}"
            )));
        };

        VarInt(self.0.len() as i32).write(w)?;
        Ok(w.write_all(self.0.as_bytes())?)
    }
}

impl Write for str {
    fn write(&self, w: &mut impl std::io::Write) -> Result<(), ErrorKind> {
        VarInt(self.len() as i32).write(w)?;
        Ok(w.write_all(self.as_bytes())?)
    }
}

#[derive(Debug)]
pub struct Rest<T, const BOUND: usize = 32767>(pub T);

impl<'a, const BOUND: usize> Read<'_> for Rest<String, BOUND> {
    fn read(r: &mut impl std::io::Read) -> Result<Self, ErrorKind> {
        let mut buf = Vec::new();
        r.read_to_end(&mut buf)?;

        let content = String::from_utf8(buf)
            .map_err(|_| InvalidData("Rest was not valid UTF-8".to_string()))?;

        let utf16_len = content.encode_utf16().count();

        if utf16_len > BOUND {
            return Err(InvalidData(format!(
                "utf-16 encoded string exceeds {BOUND} chars (is {utf16_len})"
            )));
        };

        Ok(Rest(content))
    }
}

impl<'a, const BOUND: usize> Write for Rest<String, BOUND> {
    fn write(&self, w: &mut impl std::io::Write) -> Result<(), ErrorKind> {
        let len = self.0.encode_utf16().count();

        if len > BOUND {
            return Err(InvalidData(format!(
                "length of string {len} exceeds bound {BOUND}"
            )));
        };

        Ok(w.write_all(self.0.as_bytes())?)
    }
}
