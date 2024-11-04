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

use bytes::{Buf, BytesMut};
use color_eyre::eyre::{bail, ensure, Context, Result};

use crate::protocol::{Decode, MAX_PACKET_SIZE};

use super::{
    datatypes::{VarInt, VariableDecodeError, VariableNumber},
    ServerboundPacket,
};

#[cfg(feature = "encryption")]
type _Cipher = cfb8::Decryptor<aes::Aes128>;

#[derive(Default, Debug)]
pub struct Decoder {
    buf: BytesMut,
    #[cfg(feature = "encryption")]
    _compression_threshold: i32,
}

#[derive(Debug, Clone)]
pub struct Frame {
    pub id: i32,
    pub body: BytesMut,
}

impl Frame {
    pub fn decode<'a, P>(&'a self) -> Result<P>
    where
        P: ServerboundPacket<'a>,
    {
        ensure!(
            P::ID == self.id,
            "Mismatched packet IDs: expected {} got {}",
            P::ID,
            self.id
        );

        let mut r = &self.body[..];
        let p = P::decode(&mut r)?;

        ensure!(
            r.is_empty(),
            "Didn't decode enough bytes decoding {}: {} left",
            P::ID,
            r.len()
        );

        Ok(p)
    }
}

impl Decoder {
    #[must_use]
    pub fn new() -> Self {
        Self {
            buf: BytesMut::default(),
            #[cfg(feature = "encryption")]
            _compression_threshold: -1, // disabled
        }
    }

    pub fn reserve_additional(&mut self, additional: usize) {
        self.buf.reserve(additional);
    }

    pub fn try_read_next(&mut self) -> Result<Option<Frame>> {
        let mut buf = &self.buf[..];

        let len = match VarInt::decode(&mut buf) {
            Ok(l) => l,
            Err(e) => match e.downcast_ref::<VariableDecodeError>() {
                Some(&VariableDecodeError::Incomplete) => {
                    trace!("Incomplete packet {:#?}", &buf);
                    return Ok(None);
                }
                Some(&VariableDecodeError::TooLong) => bail!("Invalid packet length"),
                None => bail!("Unknown error: {e}"),
            },
        };

        ensure!(
            0 <= len.0 && len.0 <= MAX_PACKET_SIZE,
            "Packet length {len} is out of bounds (min 0, max {MAX_PACKET_SIZE})",
        );

        if buf.len() < len.0 as usize {
            // packet is incomplete, keep waiting
            return Ok(None);
        }

        // TODO: use compression here
        self.buf.advance(len.len());
        let mut data = self.buf.split_to(len.0 as usize);
        buf = &data[..];

        let packet_id = VarInt::decode(&mut buf)
            .context("Failed to decode packet ID")?
            .0;

        // advance to end of packet id
        data.advance(data.len() - buf.len());

        Ok(Some(Frame {
            id: packet_id,
            body: data,
        }))
    }

    pub fn take_all(&mut self) -> BytesMut {
        self.buf.split_off(self.buf.len())
    }

    pub fn add_bytes(&mut self, bytes: BytesMut) {
        self.buf.unsplit(bytes);
    }
}
