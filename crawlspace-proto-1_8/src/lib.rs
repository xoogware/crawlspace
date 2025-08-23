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

use bytes::BytesMut;
use crawlspace_proto::{
    Packet, Read, ServerboundPacket,
    datatypes::{VarInt, VariableNumber},
};

/// Minecraft versions 1.8-1.8.9
/// Protocol version 47
pub struct Protocol47<R, W> {
    reader: R,
    writer: W,
    bytebuf: BytesMut,
}

impl<R: std::io::Read, W: std::io::Write> Protocol47<R, W> {
    pub fn new(reader: R, writer: W) -> Self {
        Self {
            reader,
            writer,
            bytebuf: BytesMut::new(),
        }
    }

    fn read_packet(&mut self) -> Result<Box<dyn ServerboundPacket>, crawlspace_proto::ErrorKind> {
        let len = VarInt::read(&mut self.reader)?;

        todo!();
    }
}

impl<R: std::io::Read, W: std::io::Write> crawlspace_proto::Protocol for Protocol47<R, W> {
    fn handshake_player(&mut self) {}
}
