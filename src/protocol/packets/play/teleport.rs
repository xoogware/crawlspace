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
    velocity_x: f64,
    velocity_y: f64,
    velocity_z: f64,
    yaw: f32,
    pitch: f32,
    flags: i32,
    pub id: i32,
}

#[allow(unused)]
mod flags {
    pub const X: i32 = 0x01;
    pub const Y: i32 = 0x02;
    pub const Z: i32 = 0x04;
    pub const Y_ROT: i32 = 0x08;
    pub const X_ROT: i32 = 0x10;
    pub const REL_VEL_X: i32 = 0x20;
    pub const REL_VEL_Y: i32 = 0x40;
    pub const REL_VEL_Z: i32 = 0x80;
    pub const ROTATE_VEL: i32 = 0x100;
}

#[allow(unused)]
impl SynchronisePositionC {
    pub fn new(
        x: f64,
        y: f64,
        z: f64,
        velocity_x: f64,
        velocity_y: f64,
        velocity_z: f64,
        yaw: f32,
        pitch: f32
    ) -> Self {
        Self {
            x,
            y,
            z,
            velocity_x,
            velocity_y,
            velocity_z,
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

    pub const fn relative_velocity_x(mut self) -> Self {
        self.flags |= flags::REL_VEL_X;
        self
    }

    pub const fn relative_velocity_y(mut self) -> Self {
        self.flags |= flags::REL_VEL_Y;
        self
    }

    pub const fn relative_velocity_z(mut self) -> Self {
        self.flags |= flags::REL_VEL_Z;
        self
    }

    pub const fn rotate_velocity(mut self) -> Self {
        self.flags |= flags::ROTATE_VEL;
        self
    }
}

impl Packet for SynchronisePositionC {
    const ID: i32 = 0x42;
}

impl Encode for SynchronisePositionC {
    fn encode(&self, mut w: impl std::io::Write) -> color_eyre::eyre::Result<()> {
        VarInt(self.id).encode(&mut w)?;
        self.x.encode(&mut w)?;
        self.y.encode(&mut w)?;
        self.z.encode(&mut w)?;
        self.velocity_x.encode(&mut w)?;
        self.velocity_y.encode(&mut w)?;
        self.velocity_z.encode(&mut w)?;
        self.yaw.encode(&mut w)?;
        self.pitch.encode(&mut w)?;
        self.flags.encode(&mut w)?;

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
