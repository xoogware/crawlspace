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

#[allow(unused_imports)]
pub mod datatypes {
    mod impls;
    mod position;
    mod slot;
    mod string;
    mod text_component;
    mod variable;

    pub use impls::*;
    pub use position::*;
    pub use slot::*;
    pub use string::*;
    pub use text_component::*;
    pub use variable::*;
}

pub enum PacketState {
    Handshake,
    Play,
    Status,
    Login,
}

#[derive(thiserror::Error, Debug)]
pub enum ErrorKind {
    #[error("IO error")]
    Io(#[from] std::io::Error),
    #[error("Invalid data: {0}")]
    InvalidData(String),
}

pub trait Read<'a> {
    fn read(reader: &mut impl std::io::Read) -> Result<Self, ErrorKind>
    where
        Self: Sized;
}

pub trait Write {
    fn write(&self, writer: &mut impl std::io::Write) -> Result<(), ErrorKind>;
}

pub trait Packet {
    fn packet_id(&self) -> &'static str;
    fn packet_state(&self) -> PacketState;
}

pub trait ServerboundPacket: Packet + for<'a> Read<'a> {}
pub trait ClientboundPacket: Packet + Write {}

pub trait Protocol {
    fn handshake_player(&mut self);
    // fn login_player(&self);
}
