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
use registry::Registry;
// use registry::Registry;
use serde_json::json;
use thiserror::Error;
use tokio::{
    net::TcpStream,
    sync::{Mutex, OwnedSemaphorePermit, RwLock},
    time::{self, timeout},
};

use crate::{
    protocol::{
        datatypes::{Bounded, VarInt},
        packets::{
            login::*,
            play::{ConfirmTeleportS, Gamemode, LoginPlayC, SynchronisePositionC},
        },
        PacketState,
    },
    CrawlState,
};

#[cfg(feature = "encryption")]
use crate::protocol::{datatypes::Bytes, packets::PluginRequestC};

use super::io::NetIo;

#[derive(Debug)]
pub struct Player {
    pub id: u16,
    _permit: OwnedSemaphorePermit,
    io: Mutex<NetIo>,

    crawlstate: CrawlState,
    packet_state: RwLock<PacketState>,

    tp_state: Mutex<TeleportState>,
}

#[derive(Debug)]
enum TeleportState {
    Pending(i32, time::Instant),
    Clear,
}

#[derive(Clone, Debug)]
pub struct SharedPlayer(Arc<Player>);

impl SharedPlayer {
    #[must_use]
    pub fn new(
        crawlstate: CrawlState,
        permit: OwnedSemaphorePermit,
        id: u16,
        connection: TcpStream,
    ) -> Self {
        Self(Arc::new(Player {
            id,
            io: Mutex::new(NetIo::new(connection)),
            _permit: permit,

            crawlstate,
            packet_state: RwLock::new(PacketState::Handshaking),

            tp_state: Mutex::new(TeleportState::Clear),
        }))
    }

    #[inline(always)]
    pub fn id(&self) -> u16 {
        self.0.id
    }

    pub async fn connect(&self) {
        {
            let io = self.0.io.lock().await;
            let addy = io
                .peer_addr()
                .map_or("Unknown".to_string(), |a| a.to_string());
            debug!("Handling new player (id {}) from {}", self.0.id, addy);
        }

        // crawlspace intentionally doesn't support legacy pings :3
        match timeout(Duration::from_secs(5), self.handshake()).await {
            Err(e) => warn!("Timed out waiting for {} to connect: {e}", self.0.id),
            Ok(Err(why)) => warn!("Error handshaking: {why}"),
            Ok(Ok(())) => {
                let s = self.0.packet_state.read().await;
                if let PacketState::Status = *s {
                    return;
                }
                drop(s);

                debug!(
                    "Handshake complete for client {}. Starting play loop.",
                    self.0.id
                );

                match self.begin_play().await {
                    Ok(()) => debug!("Play loop for {} done.", self.id()),
                    Err(why) => error!("Failed to play player {}! {why}", self.id()),
                }
            }
        }
    }

    async fn handshake(&self) -> Result<()> {
        let state = self.0.crawlstate.clone();
        let mut io = self.0.io.lock().await;

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
            }
            s => unimplemented!("state {:#?} unimplemented after handshake", s),
        }

        Ok(())
    }

    async fn handle_status(&self) -> Result<()> {
        let mut io = self.0.io.lock().await;

        io.rx::<StatusRequestS>().await?;
        let state = self.0.crawlstate.clone();

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

    async fn login(&self) -> Result<()> {
        let state = self.0.crawlstate.clone();
        let mut io = self.0.io.lock().await;

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

        let registry = &*registry::ALL_REGISTRIES;
        io.tx(&Registry::from(registry.trim_material.clone()))
            .await?;
        io.tx(&Registry::from(registry.trim_pattern.clone()))
            .await?;
        io.tx(&Registry::from(registry.banner_pattern.clone()))
            .await?;
        io.tx(&Registry::from(registry.biome.clone())).await?;
        io.tx(&Registry::from(registry.chat_type.clone())).await?;
        io.tx(&Registry::from(registry.damage_type.clone())).await?;
        io.tx(&Registry::from(registry.dimension_type.clone()))
            .await?;
        io.tx(&Registry::from(registry.wolf_variant.clone()))
            .await?;
        io.tx(&Registry::from(registry.painting_variant.clone()))
            .await?;

        io.tx(&FinishConfigurationC).await?;
        io.rx::<FinishConfigurationAckS>().await?;

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

    async fn begin_play(&self) -> Result<()> {
        let mut packet_state = self.0.packet_state.write().await;
        *packet_state = PacketState::Play;

        let state = self.0.crawlstate.clone();
        let mut io = self.0.io.lock().await;

        let max_players: i32 = state.max_players.try_into().unwrap_or(50);

        let login = LoginPlayC {
            entity_id: self.0.id as i32,
            is_hardcore: false,
            dimension_names: vec![Bounded::<&'static str>("minecraft:the_end")],
            max_players: VarInt(max_players),
            view_distance: VarInt(32),
            simulation_distance: VarInt(8),
            reduced_debug_info: !cfg!(debug_assertions),
            enable_respawn_screen: false,
            do_limited_crafting: false,
            dimension_type: VarInt(2),
            dimension_name: Bounded::<&'static str>("minecraft:the_end"),
            hashed_seed: 0,
            gamemode: Gamemode::Adventure,
            previous_gamemode: Some(Gamemode::Adventure),
            is_debug: false,
            is_superflat: false,
            death_location: None,
            portal_cooldown: VarInt(0),
            enforces_secure_chat: false,
        };

        io.tx(&login).await?;

        let tp = SynchronisePositionC::new(0.0, 10.0, 0.0, 0.0, 0.0);
        {
            let mut tp_state = self.0.tp_state.lock().await;
            // player will be given 5 (FIVE) SECONDS TO ACK!!!!!
            *tp_state = TeleportState::Pending(tp.id, time::Instant::now());
        }
        io.tx(&tp).await?;

        let tp_ack = io.rx::<ConfirmTeleportS>().await?;

        match tokio::time::timeout(Duration::from_secs(5), self.confirm_teleport(tp_ack.id)).await {
            Ok(Ok(())) => (),
            Ok(Err(why)) => {
                warn!("Spawning player {} failed: {why}", self.0.id);
                Err(why)?;
            }
            Err(why) => {
                warn!("Spawning player {} failed: {why}", self.0.id);
                Err(why)?;
            }
        }

        // FIXME: GROSS LOL?????? this should(?) change ownership of the player to the server
        // thread but realistically who knows burhhhh
        state.player_send.send(self.clone()).await?;

        loop {
            self.handle_packets().await?;
        }
    }

    async fn handle_packets(&self) -> Result<()> {
        Ok(())
    }

    async fn confirm_teleport(&self, id: i32) -> Result<(), TeleportError> {
        let tp_state = self.0.tp_state.lock().await;
        match *tp_state {
            TeleportState::Clear => Err(TeleportError::Unexpected),
            TeleportState::Pending(expected, _) => match id == expected {
                true => Ok(()),
                false => Err(TeleportError::WrongId(expected, id)),
            },
        }
    }
}

#[derive(Debug, Error)]
enum TeleportError {
    #[error("Client was not expecting a teleport acknowledgement")]
    Unexpected,
    #[error("Teleport ack has wrong ID (expected {0}, got {1})")]
    WrongId(i32, i32),
}
