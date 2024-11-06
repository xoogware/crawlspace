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

use std::sync::atomic::{AtomicI32, Ordering};

use crate::protocol::{datatypes::VarInt, Decode, Encode, Packet};

static TP_ID: AtomicI32 = AtomicI32::new(0);

#[derive(Debug)]
pub struct SynchronisePositionC {
    x: f64,
    y: f64,
    z: f64,
    yaw: f32,
    pitch: f32,
    flags: i8,
    pub id: i32,
}

mod flags {
    pub const X: i8 = 0x01;
    pub const Y: i8 = 0x02;
    pub const Z: i8 = 0x04;
    pub const Y_ROT: i8 = 0x08;
    pub const X_ROT: i8 = 0x10;
}

impl SynchronisePositionC {
    pub fn new(x: f64, y: f64, z: f64, yaw: f32, pitch: f32) -> Self {
        Self {
            x,
            y,
            z,
            yaw,
            pitch,
            flags: 0,
            id: TP_ID.fetch_add(1, Ordering::Relaxed),
        }
    }

    pub const fn relative_x(mut self) -> Self {
        self.flags |= flags::X;
        self
    }

    pub const fn relative_y(mut self) -> Self {
        self.flags |= flags::Y;
        self
    }

    pub const fn relative_z(mut self) -> Self {
        self.flags |= flags::Z;
        self
    }

    pub const fn relative_pitch(mut self) -> Self {
        self.flags |= flags::Y_ROT;
        self
    }

    pub const fn relative_yaw(mut self) -> Self {
        self.flags |= flags::X_ROT;
        self
    }
}

impl Packet for SynchronisePositionC {
    const ID: i32 = 0x40;
}

impl Encode for SynchronisePositionC {
    fn encode(&self, mut w: impl std::io::Write) -> color_eyre::eyre::Result<()> {
        self.x.encode(&mut w)?;
        self.y.encode(&mut w)?;
        self.z.encode(&mut w)?;
        self.yaw.encode(&mut w)?;
        self.pitch.encode(&mut w)?;
        self.flags.encode(&mut w)?;
        VarInt(self.id).encode(&mut w)?;

        Ok(())
    }
}

#[derive(Debug)]
pub struct ConfirmTeleportS {
    pub id: i32,
}

impl Packet for ConfirmTeleportS {
    const ID: i32 = 0x00;
}

impl Decode<'_> for ConfirmTeleportS {
    fn decode(r: &mut &'_ [u8]) -> color_eyre::eyre::Result<Self> {
        Ok(Self {
            id: VarInt::decode(r)?.0,
        })
    }
}
