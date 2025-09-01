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

use std::time::Duration;

use bytes::{Buf, BytesMut};
use crawlspace_proto::{
    ConnectionState, ErrorKind, Packet, Read, ServerboundPacket,
    datatypes::{VarInt, VariableNumber},
};
use handshake::HandshakeS;
use tokio::{
    io::AsyncReadExt,
    net::tcp::{OwnedReadHalf, OwnedWriteHalf},
};
use tracing::{debug, warn};

mod handshake;

const BUF_SIZE: usize = 4096;
const MAX_PACKET_SIZE: i32 = 2097152;

type Frame = (i32, BytesMut);

/// Minecraft versions 1.8-1.8.9
/// Protocol version 47
pub struct Protocol47 {
    connection_state: ConnectionState,
    read_half: OwnedReadHalf,
    write_half: OwnedWriteHalf,
    read_buf: BytesMut,
}

impl Protocol47 {
    pub fn new(read_half: OwnedReadHalf, write_half: OwnedWriteHalf) -> Self {
        Self {
            connection_state: ConnectionState::Handshake,
            read_half,
            write_half,
            read_buf: BytesMut::new(),
        }
    }

    pub async fn await_packet<T: ServerboundPacket>(&mut self) -> crawlspace_proto::Result<T> {
        // TODO: skip decoding for frames we're not looking for
        loop {
            let frame = self.read_frame().await?;

            if T::packet_id() == frame.0 {
                return Ok(T::read(&mut &frame.1[..])?);
            }

            debug!(
                "discarding packet with id {} while awaiting {:?}",
                frame.0,
                T::packet_id()
            );
        }
    }

    pub async fn read_frame(&mut self) -> crawlspace_proto::Result<Frame> {
        // TODO: maybe move this somewhere else? i don't know if a global timeout of 5 seconds per
        // packet is realistic but for testing it's chill i suppose
        tokio::time::timeout(Duration::from_secs(5), async move {
            loop {
                if let Some(frame) = self.try_read_next()? {
                    return Ok(frame);
                };

                // otherwise, keep reading the rest of the packet
                // (commented for my own sanity - these method names are awful)

                // reserve more space
                self.read_buf.reserve(BUF_SIZE);
                // split into two parts - self.read_buf containing already read data
                // and a new buf to fill with new data (.len returns number of bytes already held,
                // not capacity)
                let mut buf = self.read_buf.split_off(self.read_buf.len());

                // fills the remainder of the buf (just newly allocated space)
                if self.read_half.read_buf(&mut buf).await? == 0 {
                    return Err(std::io::Error::from(std::io::ErrorKind::UnexpectedEof).into());
                }

                // joins "bufs" back together (just moves end pointer)
                self.read_buf.unsplit(buf);
            }
        })
        .await
        .map_err(|_| ErrorKind::Timeout)?
    }

    /// Ok(None) represents an incomplete but correctly formed packet
    fn try_read_next(&mut self) -> crawlspace_proto::Result<Option<Frame>> {
        let mut buf = &self.read_buf[..];

        let len = VarInt::read(&mut buf)?;

        if len.0 < 0 || len.0 > MAX_PACKET_SIZE {
            return Err(ErrorKind::InvalidData(format!(
                "Packet length {len} is out of bounds (min 0, max {MAX_PACKET_SIZE})"
            )));
        };

        if buf.len() < len.0 as usize {
            // packet is incomplete, keep waiting
            return Ok(None);
        }

        // TODO: use compression here
        self.read_buf.advance(len.len());
        let mut data = self.read_buf.split_to(len.0 as usize);
        buf = &data[..];

        let packet_id = VarInt::read(&mut buf)?.0;

        // advance to end of packet id
        data.advance(data.len() - buf.len());

        Ok(Some((packet_id, data)))
    }
}
