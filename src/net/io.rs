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

use std::{io::ErrorKind, time::Duration};

use bytes::BytesMut;
use color_eyre::eyre::{bail, Context, Result};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{
        tcp::{OwnedReadHalf, OwnedWriteHalf},
        TcpStream,
    },
    sync::{Mutex, RwLock},
};

use crate::protocol::{self, ClientboundPacket, Frame, ServerboundPacket};

#[derive(Debug)]
pub struct NetIo {
    pub peer_addr: String,
    pub connected: RwLock<bool>,
    read_half: Mutex<OwnedReadHalf>,
    write_half: Mutex<OwnedWriteHalf>,
    decoder: Mutex<protocol::Decoder>,
    encoder: Mutex<protocol::Encoder>,
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

        let peer_addr = stream
            .peer_addr()
            .map_or("Unknown".to_owned(), |a| a.to_string());
        let (read_half, write_half) = stream.into_split();

        Self {
            peer_addr,
            connected: RwLock::new(true),
            read_half: Mutex::new(read_half),
            write_half: Mutex::new(write_half),
            decoder: Mutex::new(protocol::Decoder::new()),
            encoder: Mutex::new(protocol::Encoder::new()),
        }
    }

    pub async fn connected(&self) -> bool {
        let c = self.connected.read().await;
        *c
    }

    pub async fn rx<'a, 'b, P>(&'a self) -> Result<Frame>
    where
        P: ServerboundPacket<'a>,
    {
        // TODO: maybe move this somewhere else? i don't know if a global timeout of 5 seconds per
        // packet is realistic but for testing it's chill i suppose
        tokio::time::timeout(Duration::from_secs(5), async move {
            let mut decoder = self.decoder.lock().await;
            loop {
                if let Some(frame) = decoder.try_read_next().context("failed try_read_next")? {
                    if frame.id != P::ID {
                        debug!(
                            "Got packet ID {} while awaiting {}, discarding",
                            frame.id,
                            P::ID
                        );
                        continue;
                    }

                    // TODO: decode here, rather than forcing the consumer to do it.
                    // probably need to box frame data? idk enough rust for this
                    return Ok(frame);
                };

                decoder.reserve_additional(BUF_SIZE);
                let mut buf = decoder.take_all();

                let mut read_half = self.read_half.lock().await;
                if read_half
                    .read_buf(&mut buf)
                    .await
                    .context("failed read_buf")?
                    == 0
                {
                    let mut c = self.connected.write().await;
                    *c = false;
                    return Err(std::io::Error::from(ErrorKind::UnexpectedEof).into());
                }

                decoder.add_bytes(buf);
            }
        })
        .await?
    }

    pub async fn tx<P>(&self, packet: &P) -> Result<()>
    where
        P: ClientboundPacket,
    {
        trace!("Sending packet {:?}", packet);
        let mut encoder = self.encoder.lock().await;
        encoder.append_packet(packet)?;
        let bytes = encoder.take();
        trace!("raw packet is {} bytes", bytes.len());
        trace!("{:?}", bytes.to_vec());
        let mut writer = self.write_half.lock().await;
        Ok(writer.write_all(&bytes).await?)
    }

    pub async fn tx_raw(&self, packet: &[u8]) -> Result<()> {
        trace!("Sending packet {:?}", packet);
        let mut writer = self.write_half.lock().await;
        Ok(writer.write_all(packet).await?)
    }

    pub async fn rx_raw(&self) -> Result<Frame> {
        let mut decoder = self.decoder.lock().await;
        if let Some(frame) = decoder.try_read_next().context("failed try_read_next")? {
            return Ok(frame);
        };

        decoder.reserve_additional(BUF_SIZE);
        let mut buf = decoder.take_all();

        {
            let mut reader = self.read_half.lock().await;
            if reader.read_buf(&mut buf).await.context("failed read_buf")? == 0 {
                let mut c = self.connected.write().await;
                *c = false;
                return Err(std::io::Error::from(ErrorKind::UnexpectedEof).into());
            }
        }

        decoder.add_bytes(buf);

        bail!("No packet available")
    }
}
