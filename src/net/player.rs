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

use std::{sync::Arc, time::Duration};

use color_eyre::eyre::Result;
use serde_json::json;
use tokio::{
    net::TcpStream,
    sync::{Mutex, OwnedSemaphorePermit, RwLock},
    time::timeout,
};

use crate::{
    protocol::{
        datatypes::{Bounded, VarInt},
        packets::{
            login::{
                FinishConfigurationAckS, FinishConfigurationC, HandshakeS, KnownPacksC,
                KnownPacksS, LoginAckS, LoginStartS, LoginSuccessC, Ping, StatusRequestS,
                StatusResponseC,
            },
            play::LoginPlayC,
        },
        PacketState,
    },
    CrawlState,
};

#[cfg(feature = "encryption")]
use crate::protocol::{datatypes::Bytes, packets::PluginRequestC};

use super::io::NetIo;

#[derive(Clone, Debug)]
pub struct Player {
    pub id: u16,
    _permit: Arc<OwnedSemaphorePermit>,
    io: Arc<Mutex<NetIo>>,

    crawlstate: CrawlState,
    packet_state: Arc<RwLock<PacketState>>,
}

impl Player {
    #[must_use]
    pub fn new(
        crawlstate: CrawlState,
        permit: OwnedSemaphorePermit,
        id: u16,
        connection: TcpStream,
    ) -> Self {
        Self {
            id,
            io: Arc::new(Mutex::new(NetIo::new(connection))),
            _permit: Arc::new(permit),

            crawlstate,
            packet_state: Arc::new(RwLock::new(PacketState::Handshaking)),
        }
    }

    pub async fn connect(&mut self) {
        let io = self.io.clone();
        let io = io.lock().await;
        let addy = io
            .peer_addr()
            .map_or("Unknown".to_string(), |a| a.to_string());
        drop(io);

        debug!("Handling new player (id {}) from {}", self.id, addy);

        // crawlspace intentionally doesn't support legacy pings :3
        match timeout(Duration::from_secs(5), self.handshake()).await {
            Err(e) => warn!("Timed out waiting for {} to connect: {e}", self.id),
            Ok(Err(why)) => warn!("Error handshaking: {why}"),
            Ok(Ok(())) => debug!("Handshake complete for client {}.", self.id),
        }
    }

    async fn handshake(&mut self) -> Result<()> {
        let state = self.crawlstate.clone();
        let io = self.io.clone();
        let mut io = io.lock().await;

        let p = io.rx::<HandshakeS>().await?;

        if p.protocol_version.0 != state.version_number {
            warn!(
                "Client protocol version {} doesn't match server version {}!",
                p.protocol_version.0, state.version_number
            );
        }

        let next_state = p.next_state;

        drop(io);

        match next_state {
            PacketState::Status => {
                self.handle_status().await?;
            }
            PacketState::Login => {
                self.login().await?;
                self.begin_play().await?;
            }
            s => unimplemented!("state {:#?} unimplemented after handshake", s),
        }

        Ok(())
    }

    async fn handle_status(&mut self) -> Result<()> {
        let io = self.io.clone();
        let mut io = io.lock().await;

        io.rx::<StatusRequestS>().await?;
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

        io.tx(&res).await?;
        let ping = io.rx::<Ping>().await?;
        io.tx(&ping).await?;

        Ok(())
    }

    async fn login(&mut self) -> Result<()> {
        let state = self.crawlstate.clone();
        let io = self.io.clone();
        let mut io = io.lock().await;

        let login = io.rx::<LoginStartS>().await?;

        // need to manually clone this or else the reference to self.io lives too long
        // TODO: clean up lifetimes on encode/decode - possibly just clone strings?
        let uuid = login.player_uuid;
        let username = login.name.0.to_owned();

        #[cfg(feature = "encryption")]
        self.login_velocity(&username).await?;

        let success = LoginSuccessC {
            uuid,
            username: Bounded(&username),
            properties: Vec::new(),
            strict_error_handling: false,
        };

        io.tx(&success).await?;
        io.rx::<LoginAckS>().await?;

        let clientbound_known_packs = KnownPacksC::of_version(&state.version_name);
        io.tx(&clientbound_known_packs).await?;

        // TODO: maybe(?) actually handle this
        io.rx::<KnownPacksS>().await?;

        io.tx(&FinishConfigurationC).await?;
        io.rx::<FinishConfigurationAckS>().await?;

        Ok(())
    }

    async fn begin_play(&mut self) -> Result<()> {
        let packet_state = self.packet_state.clone();
        let packet_state = packet_state.write().await;

        let state = self.crawlstate.clone();
        let io = self.io.clone();
        let mut io = io.lock().await;

        let max_players: i32 = state.max_players.try_into().unwrap_or(50);

        let login = LoginPlayC {
            entity_id: self.id as i32,
            is_hardcore: false,
            dimension_names: vec![Bounded::<&'static str>("the_end")],
            max_players: VarInt(max_players),
            view_distance: VarInt(32),
            simulation_distance: VarInt(8),
            reduced_debug_info: cfg!(debug_assertions),
        };

        // FIXME: GROSS LOL?????? this should(?) change ownership of the player to the server
        // thread but realistically who knows burhhhh
        state.player_send.send(Arc::new(self.clone())).await?;

        Ok(())
    }

    #[cfg(feature = "encryption")]
    async fn login_velocity(&mut self, _username: &str) -> Result<()> {
        let req = PluginRequestC {
            message_id: VarInt(0),
            channel: Bounded("velocity:player_info"),
            data: Bounded(Bytes(&[3])),
        };

        self.io.tx(&req).await?;

        Ok(())
    }
}
