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

use crawlspace_macro::{Encode, Packet};

use crate::protocol::{datatypes::VarInt, Encode, Packet, PacketDirection, PacketState};

#[derive(Debug, Packet, Encode)]
#[packet(
    id = "minecraft:ticking_state",
    clientbound,
    state = "PacketState::Play"
)]
pub struct SetTickingStateC {
    pub tick_rate: f32,
    pub is_frozen: bool,
}

#[derive(Debug, Packet, Encode)]
#[packet(
    id = "minecraft:ticking_step",
    clientbound,
    state = "PacketState::Play"
)]
pub struct StepTicksC(#[varint] pub i32);
