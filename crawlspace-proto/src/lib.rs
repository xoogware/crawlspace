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

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ConnectionState {
    Handshake,
    Play,
    Status,
    Login,
}

#[derive(thiserror::Error, Debug)]
pub enum ErrorKind {
    #[error("Not connected to a client")]
    Disconnected,
    #[error("IO error")]
    Io(#[from] std::io::Error),
    #[error("Invalid data: {0}")]
    InvalidData(String),
    #[error("Timed out")]
    Timeout,
}

pub type Result<T> = std::result::Result<T, ErrorKind>;

pub trait Read {
    fn read(reader: &mut impl std::io::Read) -> Result<Self>
    where
        Self: Sized;
}

pub trait Write {
    fn write(&self, writer: &mut impl std::io::Write) -> Result<()>;
}

#[derive(Clone, Debug)]
pub enum PacketId {
    String(&'static str),
    Numeric(i32),
}

impl PartialEq<i32> for PacketId {
    fn eq(&self, other: &i32) -> bool {
        match self {
            Self::Numeric(i) => i == other,
            _ => false,
        }
    }
}

impl PartialEq<&'static str> for PacketId {
    fn eq(&self, other: &&'static str) -> bool {
        match self {
            Self::String(s) => s == other,
            _ => false,
        }
    }
}

pub trait Packet {
    fn packet_id() -> PacketId
    where
        Self: Sized;
    fn packet_state() -> ConnectionState
    where
        Self: Sized;
}

pub trait ServerboundPacket: Packet + Read {}
impl<T> ServerboundPacket for T where T: Packet + Read {}

pub trait ClientboundPacket: Packet + Write {}
impl<T> ClientboundPacket for T where T: Packet + Write {}

pub trait Protocol {
    fn handshake_player(&mut self);
    // fn login_player(&self);
}
