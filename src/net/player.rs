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

use color_eyre::eyre::Result;
use tokio::net::TcpStream;
use uuid::Uuid;

use crate::protocol::{
    packets::{HandshakeS, StatusRequestS},
    PacketState,
};

use super::io::NetIo;

pub struct Player {
    pub id: u16,
    state: PlayerState,
    io: NetIo,
}

pub enum PlayerState {
    Connecting,
    Playing(Uuid),
    Broken,
    Closed,
}

impl Player {
    #[must_use]
    pub fn new(id: u16, connection: TcpStream) -> Self {
        Self {
            id,
            state: PlayerState::Connecting,
            io: NetIo::new(connection),
        }
    }

    pub async fn connect(&mut self) {
        debug!(
            "Handling new player (id {}) from {}",
            self.id,
            self.io
                .peer_addr()
                .map_or("Unknown".to_string(), |a| a.to_string()),
        );

        // crawlspace intentionally doesn't support legacy pings :3
        match self.handshake().await {
            Ok(()) => (),
            Err(why) => {
                error!("Error handshaking: {why}");
            }
        }
    }

    async fn handshake(&mut self) -> Result<()> {
        let p = self.io.rx::<HandshakeS>().await?;
        let next_state = p.next_state;

        debug!(
            "Got handshake packet from {}; prococol version {}, next state {:#?}",
            self.id, p.protocol_version, p.next_state
        );

        match next_state {
            PacketState::Status => {
                self.handle_status().await?;
            }
            s => {
                unimplemented!("state {:#?} unimplemented after handshake", s);
            }
        }

        Ok(())
    }

    async fn handle_status(&mut self) -> Result<()> {
        let p = self.io.rx::<StatusRequestS>().await?;
        Ok(())
    }
}
