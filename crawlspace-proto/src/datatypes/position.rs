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

use bitfield_struct::bitfield;
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use thiserror::Error;

use crate::{ErrorKind, Read, Write};

#[derive(Debug)]
pub struct Position {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

#[bitfield(u64)]
pub struct PackedPosition {
    #[bits(26)]
    pub x: i32,
    #[bits(26)]
    pub z: i32,
    #[bits(12)]
    pub y: i32,
}

impl Position {
    #[must_use]
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }
}

#[derive(Debug, Error)]
pub enum EncodingError {
    #[error("value is out of bounds")]
    OutOfBounds,
}

impl TryFrom<&Position> for PackedPosition {
    type Error = EncodingError;

    fn try_from(value: &Position) -> std::result::Result<Self, Self::Error> {
        match (value.x, value.y, value.z) {
            (-0x2000000..=0x1ffffff, -0x800..=0x7ff, -0x2000000..=0x1ffffff) => {
                Ok(PackedPosition::new()
                    .with_x(value.x)
                    .with_y(value.y)
                    .with_z(value.z))
            }
            _ => Err(EncodingError::OutOfBounds),
        }
    }
}

impl From<PackedPosition> for Position {
    fn from(value: PackedPosition) -> Self {
        Self {
            x: value.x(),
            y: value.y(),
            z: value.z(),
        }
    }
}

impl Write for Position {
    fn write(&self, w: &mut impl std::io::Write) -> Result<(), ErrorKind> {
        let encoded: PackedPosition = self
            .try_into()
            .map_err(|_| ErrorKind::InvalidData("Invalid packed position".to_string()))?;
        encoded.write(w)
    }
}

impl Read<'_> for Position {
    fn read(r: &mut impl std::io::Read) -> Result<Self, ErrorKind> {
        let bytes = r.read_i64::<BigEndian>()?;

        Ok(Self {
            x: (bytes >> 38) as i32,
            y: (bytes << 52 >> 52) as i32,
            z: (bytes << 26 >> 38) as i32,
        })
    }
}

impl Write for PackedPosition {
    fn write(&self, w: &mut impl std::io::Write) -> Result<(), ErrorKind> {
        Ok(w.write_u64::<BigEndian>(self.0)?)
    }
}
