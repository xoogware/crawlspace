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

use std::{io::ErrorKind, net::SocketAddr};

use bytes::BytesMut;
use color_eyre::eyre::Result;
use tokio::{io::AsyncReadExt, net::TcpStream};

use crate::protocol::{self, Decode, Frame, ServerboundPacket};

pub struct NetIo {
    stream: TcpStream,
    frame: Frame,
    decoder: protocol::Decoder,
}

const BUF_SIZE: usize = 4096;

impl NetIo {
    #[must_use]
    pub fn new(stream: TcpStream) -> Self {
        if let Err(why) = stream.set_nodelay(true) {
            warn!(
                "Failed to set nodelay for {}: {why}",
                stream
                    .peer_addr()
                    .map_or("Unknown".to_string(), |a| a.to_string()),
            );
        }

        Self {
            stream,
            frame: Frame {
                id: -1,
                body: BytesMut::new(),
            },
            decoder: protocol::Decoder::new(),
        }
    }

    pub fn peer_addr(&self) -> Result<SocketAddr> {
        Ok(self.stream.peer_addr()?)
    }

    pub async fn rx<'a, P>(&'a mut self) -> Result<P>
    where
        P: ServerboundPacket<'a>,
    {
        loop {
            if let Some(frame) = self.decoder.try_read_next()? {
                self.frame = frame;
                let r = self.frame.decode();
                debug!("Got packet {:#?}", r);
                return r;
            };

            self.decoder.reserve_additional(BUF_SIZE);
            let mut buf = self.decoder.take_all();

            if self.stream.read_buf(&mut buf).await? == 0 {
                return Err(std::io::Error::from(ErrorKind::UnexpectedEof).into());
            }

            self.decoder.add_bytes(buf);
        }
    }
}
