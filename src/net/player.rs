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
use serde_json::json;
use tokio::net::TcpStream;
use uuid::Uuid;

use crate::{
    protocol::{
        packets::{HandshakeS, Ping, StatusRequestS, StatusResponseC},
        PacketState,
    },
    CrawlState,
};

use super::io::NetIo;

pub struct Player {
    pub id: u16,
    io: NetIo,

    crawlstate: CrawlState,
}

impl Player {
    #[must_use]
    pub fn new(crawlstate: CrawlState, id: u16, connection: TcpStream) -> Self {
        Self {
            id,
            io: NetIo::new(connection),

            crawlstate,
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

        match next_state {
            PacketState::Status => {
                self.handle_status().await?;
            }
            PacketState::Login => {}
            s => unimplemented!("state {:#?} unimplemented after handshake", s),
        }

        Ok(())
    }

    async fn handle_status(&mut self) -> Result<()> {
        self.io.rx::<StatusRequestS>().await?;
        let state = self.crawlstate.clone();

        let res = json!({
            "version": {
                "name": state.version_name,
                "protocol": state.version_number,
            },
            "players": {
                "online": state.current_players,
                "max": state.max_players
            },
            "description": {
                "text": state.description
            },
            "enforcesSecureChat": false
        });

        let res = StatusResponseC {
            json_respose: &res.to_string(),
        };

        self.io.tx(&res).await?;
        let ping = self.io.rx::<Ping>().await?;
        self.io.tx(&ping).await?;

        Ok(())
    }
}
