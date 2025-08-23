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

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use uuid::Uuid;

use crate::{
    ErrorKind::{self, InvalidData},
    Read, Write,
};

impl Read<'_> for bool {
    fn read(r: &mut impl std::io::Read) -> Result<Self, ErrorKind> {
        Ok(match r.read_u8()? {
            0x01 => true,
            0x00 => false,
            v => {
                return Err(InvalidData(format!(
                    "Expected 0x01 or 0x00 for bool, got {v}"
                )));
            }
        })
    }
}

impl Write for bool {
    fn write(&self, w: &mut impl std::io::Write) -> Result<(), ErrorKind> {
        let v = match self {
            true => 0x01,
            false => 0x00,
        };

        Ok(w.write_all(&[v])?)
    }
}

impl Read<'_> for i8 {
    fn read(r: &mut impl std::io::Read) -> Result<Self, ErrorKind> {
        Ok(r.read_i8()?)
    }
}

impl Write for i8 {
    fn write(&self, w: &mut impl std::io::Write) -> Result<(), ErrorKind> {
        Ok(w.write_i8(*self)?)
    }
}

impl Read<'_> for u8 {
    fn read(r: &mut impl std::io::Read) -> Result<Self, ErrorKind>
    where
        Self: Sized,
    {
        Ok(r.read_u8()?)
    }
}

impl Write for u8 {
    fn write(&self, w: &mut impl std::io::Write) -> Result<(), ErrorKind> {
        Ok(w.write_u8(*self)?)
    }
}

impl Read<'_> for i16 {
    fn read(r: &mut impl std::io::Read) -> Result<Self, ErrorKind>
    where
        Self: Sized,
    {
        Ok(r.read_i16::<BigEndian>()?)
    }
}

impl Write for i16 {
    fn write(&self, w: &mut impl std::io::Write) -> Result<(), ErrorKind> {
        Ok(w.write_i16::<BigEndian>(*self)?)
    }
}

impl Read<'_> for u16 {
    fn read(r: &mut impl std::io::Read) -> Result<Self, ErrorKind>
    where
        Self: Sized,
    {
        Ok(r.read_u16::<BigEndian>()?)
    }
}

impl Read<'_> for i32 {
    fn read(r: &mut impl std::io::Read) -> Result<Self, ErrorKind>
    where
        Self: Sized,
    {
        Ok(r.read_i32::<BigEndian>()?)
    }
}

impl Write for i32 {
    fn write(&self, w: &mut impl std::io::Write) -> Result<(), ErrorKind> {
        Ok(w.write_i32::<BigEndian>(*self)?)
    }
}

impl Read<'_> for i64 {
    fn read(r: &mut impl std::io::Read) -> Result<Self, ErrorKind>
    where
        Self: Sized,
    {
        Ok(r.read_i64::<BigEndian>()?)
    }
}

impl Write for i64 {
    fn write(&self, w: &mut impl std::io::Write) -> Result<(), ErrorKind> {
        Ok(w.write_i64::<BigEndian>(*self)?)
    }
}

impl Read<'_> for u64 {
    fn read(r: &mut impl std::io::Read) -> Result<Self, ErrorKind>
    where
        Self: Sized,
    {
        Ok(r.read_u64::<BigEndian>()?)
    }
}

impl Write for u64 {
    fn write(&self, w: &mut impl std::io::Write) -> Result<(), ErrorKind> {
        Ok(w.write_u64::<BigEndian>(*self)?)
    }
}

impl Read<'_> for u128 {
    fn read(r: &mut impl std::io::Read) -> Result<Self, ErrorKind>
    where
        Self: Sized,
    {
        Ok(r.read_u128::<BigEndian>()?)
    }
}

impl Write for u128 {
    fn write(&self, w: &mut impl std::io::Write) -> Result<(), ErrorKind> {
        Ok(w.write_u128::<BigEndian>(*self)?)
    }
}

impl Read<'_> for f32 {
    fn read(r: &mut impl std::io::Read) -> Result<Self, ErrorKind>
    where
        Self: Sized,
    {
        Ok(r.read_f32::<BigEndian>()?)
    }
}

impl Write for f32 {
    fn write(&self, w: &mut impl std::io::Write) -> Result<(), ErrorKind> {
        Ok(w.write_f32::<BigEndian>(*self)?)
    }
}

impl Read<'_> for f64 {
    fn read(r: &mut impl std::io::Read) -> Result<Self, ErrorKind>
    where
        Self: Sized,
    {
        Ok(r.read_f64::<BigEndian>()?)
    }
}

impl Write for f64 {
    fn write(&self, w: &mut impl std::io::Write) -> Result<(), ErrorKind> {
        Ok(w.write_f64::<BigEndian>(*self)?)
    }
}

impl Write for Uuid {
    fn write(&self, w: &mut impl std::io::Write) -> Result<(), ErrorKind> {
        self.as_u128().write(w)
    }
}

impl Read<'_> for Uuid {
    fn read(r: &mut impl std::io::Read) -> Result<Self, ErrorKind> {
        Ok(Uuid::from_u128(r.read_u128::<BigEndian>()?))
    }
}

impl<T> Write for Option<T>
where
    T: Write,
{
    fn write(&self, w: &mut impl std::io::Write) -> Result<(), ErrorKind> {
        match self {
            None => Ok(()),
            Some(v) => v.write(w),
        }
    }
}

impl<T> Write for Vec<T>
where
    T: Write,
{
    fn write(&self, w: &mut impl std::io::Write) -> Result<(), ErrorKind> {
        for item in self {
            item.write(w)?;
        }

        Ok(())
    }
}

impl<'a, T> Read<'a> for Vec<T>
where
    T: Read<'a>,
{
    fn read(r: &mut impl std::io::Read) -> Result<Self, ErrorKind> {
        let times = r.read_i32::<BigEndian>()?;
        let mut o = Vec::new();

        for _ in 0..times {
            o.push(T::read(r)?)
        }

        Ok(o)
    }
}
