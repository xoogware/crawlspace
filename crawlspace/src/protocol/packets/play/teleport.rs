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

use crawlspace_macro::{Decode, Encode, Packet};

use crate::protocol::{datatypes::VarInt, Decode, Encode, Packet, PacketDirection, PacketState};

static TP_ID: AtomicI32 = AtomicI32::new(0);

#[derive(Debug, Packet, Encode)]
#[packet(
    id = "minecraft:player_position",
    clientbound,
    state = "PacketState::Play"
)]
pub struct SynchronisePositionC {
    #[varint]
    pub id: i32,
    x: f64,
    y: f64,
    z: f64,
    velocity_x: f64,
    velocity_y: f64,
    velocity_z: f64,
    yaw: f32,
    pitch: f32,
    flags: i32,
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
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        x: f64,
        y: f64,
        z: f64,
        velocity_x: f64,
        velocity_y: f64,
        velocity_z: f64,
        yaw: f32,
        pitch: f32,
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

#[derive(Debug, Packet, Decode)]
#[packet(
    id = "minecraft:accept_teleportation",
    serverbound,
    state = "PacketState::Play"
)]
pub struct ConfirmTeleportS {
    #[decode_as(VarInt)]
    pub id: i32,
}
