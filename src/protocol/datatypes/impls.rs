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

use std::mem;

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use color_eyre::eyre::{bail, Result};
use uuid::Uuid;

use crate::protocol::{Decode, DecodeSized, Encode};

impl<'a> Decode<'a> for bool {
    fn decode(r: &mut &'a [u8]) -> Result<Self> {
        Ok(match r.read_u8()? {
            0x01 => true,
            0x00 => false,
            v => bail!("Expected 0x01 or 0x00 for bool, got {v}"),
        })
    }
}

impl Encode for bool {
    fn encode(&self, mut w: impl std::io::Write) -> Result<()> {
        let v = match self {
            true => 0x01,
            false => 0x00,
        };

        Ok(w.write_all(&[v])?)
    }
}

impl Encode for i8 {
    fn encode(&self, mut w: impl std::io::Write) -> Result<()> {
        Ok(w.write_i8(*self)?)
    }
}

impl Encode for u8 {
    fn encode(&self, mut w: impl std::io::Write) -> Result<()> {
        Ok(w.write_u8(*self)?)
    }
}

impl Encode for i32 {
    fn encode(&self, mut w: impl std::io::Write) -> Result<()> {
        Ok(w.write_i32::<BigEndian>(*self)?)
    }
}

impl Encode for i64 {
    fn encode(&self, mut w: impl std::io::Write) -> Result<()> {
        Ok(w.write_i64::<BigEndian>(*self)?)
    }
}

impl Encode for u64 {
    fn encode(&self, mut w: impl std::io::Write) -> Result<()> {
        Ok(w.write_u64::<BigEndian>(*self)?)
    }
}

impl Encode for u128 {
    fn encode(&self, mut w: impl std::io::Write) -> Result<()> {
        Ok(w.write_u128::<BigEndian>(*self)?)
    }
}

impl Encode for Uuid {
    fn encode(&self, mut w: impl std::io::Write) -> Result<()> {
        self.as_u128().encode(&mut w)
    }
}

impl<'a> Decode<'a> for Uuid {
    fn decode(r: &mut &'a [u8]) -> Result<Self> {
        Ok(Uuid::from_u128(r.read_u128::<BigEndian>()?))
    }
}

impl<T> Encode for Option<T>
where
    T: Encode,
{
    fn encode(&self, mut w: impl std::io::Write) -> Result<()> {
        match self {
            None => Ok(()),
            Some(v) => v.encode(&mut w),
        }
    }
}

impl<T> Encode for Vec<T>
where
    T: Encode,
{
    fn encode(&self, mut w: impl std::io::Write) -> Result<()> {
        for item in self {
            item.encode(&mut w)?;
        }

        Ok(())
    }
}

impl<'a, T> DecodeSized<'a> for Vec<T>
where
    T: Decode<'a>,
{
    fn decode(times: usize, r: &mut &'a [u8]) -> Result<Self> {
        let mut o = Vec::new();

        for _ in 0..times {
            o.push(T::decode(r)?)
        }

        Ok(o)
    }
}

#[derive(Debug)]
pub struct Bytes<'a>(pub &'a [u8]);

impl<'a> Decode<'a> for Bytes<'a> {
    fn decode(r: &mut &'a [u8]) -> Result<Self> {
        Ok(Self(mem::take(r)))
    }
}

impl<'a> Encode for Bytes<'a> {
    fn encode(&self, mut w: impl std::io::Write) -> Result<()> {
        Ok(w.write_all(self.0)?)
    }
}
