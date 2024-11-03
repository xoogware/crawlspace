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

use color_eyre::eyre::{bail, Result};
use serde_json::json;
use tokio::{
    net::TcpStream,
    time::{self, sleep, timeout},
};
use uuid::Uuid;

use crate::{
    protocol::{
        datatypes::{Bounded, Bytes, Ident, VarInt},
        packets::{
            FinishConfigurationAckS, FinishConfigurationC, HandshakeS, KnownPack, KnownPacksC,
            KnownPacksS, LoginAckS, LoginStartS, LoginSuccessC, Ping, PluginRequestC,
            StatusRequestS, StatusResponseC,
        },
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
        match timeout(Duration::from_secs(5), self.handshake()).await {
            Err(e) => warn!("Timed out waiting for {} to connect: {e}", self.id),
            Ok(Err(why)) => warn!("Error handshaking: {why}"),
            Ok(Ok(())) => debug!("Handshake complete for client {}.", self.id),
        }
    }

    async fn handshake(&mut self) -> Result<()> {
        let p = self.io.rx::<HandshakeS>().await?;
        let next_state = p.next_state;

        match next_state {
            PacketState::Status => {
                self.handle_status().await?;
            }
            PacketState::Login => {
                self.login().await?;
            }
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

    async fn login(&mut self) -> Result<()> {
        let state = self.crawlstate.clone();

        let login = self.io.rx::<LoginStartS>().await?;

        // need to manually clone this or else the reference to self.io lives too long
        // TODO: clean up lifetimes on encode/decode - possibly just clone strings?
        let uuid = login.player_uuid;
        let username = login.name.0.to_owned();

        //self.login_velocity(&username).await?;

        let success = LoginSuccessC {
            uuid,
            username: Bounded(&username),
            properties: Vec::new(),
            strict_error_handling: false,
        };

        // TODO: skins
        #[cfg(feature = "skins")]
        {}

        self.io.tx(&success).await?;
        self.io.rx::<LoginAckS>().await?;

        let clientbound_known_packs = KnownPacksC::of_version(&state.version_name);
        self.io.tx(&clientbound_known_packs).await?;

        // TODO: maybe(?) actually handle this
        self.io.rx::<KnownPacksS>().await?;

        self.io.tx(&FinishConfigurationC).await?;
        self.io.rx::<FinishConfigurationAckS>().await?;

        Ok(())
    }

    async fn login_velocity(&mut self, username: &str) -> Result<()> {
        let req = PluginRequestC {
            message_id: VarInt(0),
            channel: Bounded("velocity:player_info"),
            data: Bounded(Bytes(&[3])),
        };

        self.io.tx(&req).await?;

        Ok(())
    }
}
