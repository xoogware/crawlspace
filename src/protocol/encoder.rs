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

use bytes::{BufMut, BytesMut};
use color_eyre::eyre::{ensure, Result};

use crate::protocol::{Encode, MAX_PACKET_SIZE};

use super::{
    datatypes::{VarInt, VariableNumber},
    ClientboundPacket,
};

type Cipher = cfb8::Encryptor<aes::Aes128>;

#[derive(Default)]
pub struct Encoder {
    buf: BytesMut,
}

impl Encoder {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    pub fn append(&mut self, bytes: &[u8]) {
        self.buf.extend_from_slice(bytes);
    }

    pub fn prepend_packet<P>(&mut self, packet: &P) -> Result<()>
    where
        P: ClientboundPacket,
    {
        let initial_len = self.buf.len();
        self.append_packet(packet)?;

        let after_len = self.buf.len();
        let packet_size = after_len - initial_len;

        self.buf.put_bytes(0, packet_size);
        self.buf.copy_within(..after_len, packet_size);
        self.buf.copy_within(packet_size + initial_len.., 0);
        self.buf.truncate(after_len);

        Ok(())
    }

    pub fn append_packet<P>(&mut self, packet: &P) -> Result<()>
    where
        P: ClientboundPacket,
    {
        let initial_len = self.buf.len();
        packet.encode_packet((&mut self.buf).writer())?;

        let packet_size = self.buf.len() - initial_len;

        ensure!(
            (packet_size as i32) < MAX_PACKET_SIZE,
            "packet size {packet_size} exceeds max {MAX_PACKET_SIZE}!"
        );

        let header_size = VarInt(packet_size as i32).len();

        self.buf.put_bytes(0, header_size);
        self.buf.copy_within(
            initial_len..initial_len + packet_size,
            initial_len + header_size,
        );

        let front = &mut self.buf[initial_len..];
        VarInt(packet_size as i32).encode(front)?;

        Ok(())
    }

    pub fn take(&mut self) -> BytesMut {
        self.buf.split()
    }
}
