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

use std::{
    sync::{
        atomic::{AtomicU8, Ordering},
        Arc,
    },
    time::Duration,
};

use color_eyre::eyre::{bail, Result};
use rand::Rng;
use serde_json::json;
use thiserror::Error;
use tokio::{
    net::TcpStream,
    sync::{Mutex, OwnedSemaphorePermit, RwLock},
    time::{timeout, Instant},
};
use uuid::Uuid;

use crate::{
    protocol::{
        datatypes::{Bounded, VarInt},
        packets::{
            login::*,
            play::{
                ConfirmTeleportS, GameEvent, GameEventC, Gamemode, KeepAliveC, LoginPlayC,
                OpenScreenC, PlayerInfoUpdateC, PlayerStatus, SetBorderCenterC, SetBorderSizeC,
                SetCenterChunkC, SetPlayerPositionAndRotationS, SetPlayerPositionS,
                SetTickingStateC, StepTicksC, SynchronisePositionC, UseItemOnS,
            },
        },
        Frame, Packet, PacketState,
    },
    server::window::{Window, WindowType},
    CrawlState,
};

#[cfg(feature = "encryption")]
use crate::protocol::{datatypes::Bytes, packets::login::PluginRequestC};

use super::{entity::Entity, io::NetIo};

#[derive(Debug)]
pub struct Player {
    pub id: u16,
    _permit: OwnedSemaphorePermit,
    pub io: NetIo,
    frame_queue: Mutex<Vec<Frame>>,

    crawlstate: CrawlState,
    packet_state: RwLock<PacketState>,

    uuid: RwLock<Option<Uuid>>,
    tp_state: RwLock<TeleportState>,

    last_keepalive: RwLock<Instant>,

    entity: RwLock<Entity>,

    next_window_id: AtomicU8,
    window: RwLock<Option<Window>>,
}

#[derive(Debug, PartialEq)]
enum TeleportState {
    Pending(i32, Instant),
    Clear,
}

#[derive(Clone, Debug)]
pub struct SharedPlayer(pub Arc<Player>);

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
            io: NetIo::new(connection),
            frame_queue: Mutex::new(Vec::new()),
            _permit: permit,

            crawlstate,
            packet_state: RwLock::new(PacketState::Handshaking),

            uuid: RwLock::new(None),
            tp_state: RwLock::new(TeleportState::Clear),

            last_keepalive: RwLock::new(Instant::now()),

            entity: RwLock::new(Entity::default()),

            next_window_id: AtomicU8::new(0),
            window: RwLock::new(None),
        }))
    }

    #[inline(always)]
    pub fn id(&self) -> u16 {
        self.0.id
    }

    pub async fn connect(&self) {
        {
            debug!(
                "Handling new player (id {}) from {}",
                self.0.id, self.0.io.peer_addr
            );
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

        let p = self.0.io.rx::<HandshakeS>().await?;
        let p = p.decode::<HandshakeS>()?;

        if p.protocol_version.0 != state.version_number {
            warn!(
                "Client protocol version {} doesn't match server version {}!",
                p.protocol_version.0, state.version_number
            );
        }

        let next_state = p.next_state;

        let mut s = self.0.packet_state.write().await;
        match next_state {
            PacketState::Status => {
                *s = PacketState::Status;
                drop(s);
                self.handle_status().await?;
            }
            PacketState::Login => {
                *s = PacketState::Login;
                drop(s);
                self.login().await?;
            }
            s => unimplemented!("state {:#?} unimplemented after handshake", s),
        }

        Ok(())
    }

    async fn handle_status(&self) -> Result<()> {
        self.0.io.rx::<StatusRequestS>().await?;
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

        self.0.io.tx(&res).await?;
        let ping: Ping = self.0.io.rx::<Ping>().await?.decode()?;

        self.0.io.tx(&ping).await?;

        Ok(())
    }

    async fn login(&self) -> Result<()> {
        let state = self.0.crawlstate.clone();

        let login = self.0.io.rx::<LoginStartS>().await?;
        let login: LoginStartS = login.decode()?;

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

        {
            let mut own_uuid = self.0.uuid.write().await;
            *own_uuid = Some(uuid);
        }

        self.0.io.tx(&success).await?;
        self.0.io.rx::<LoginAckS>().await?;

        let clientbound_known_packs = KnownPacksC::of_version(&state.version_name);
        self.0.io.tx(&clientbound_known_packs).await?;

        // TODO: maybe(?) actually handle this
        self.0.io.rx::<KnownPacksS>().await?;

        self.0.io.tx_raw(&state.registry_cache.encoded).await?;

        self.0.io.tx(&FinishConfigurationC).await?;
        self.0.io.rx::<FinishConfigurationAckS>().await?;

        Ok(())
    }

    #[cfg(feature = "encryption")]
    async fn login_velocity(&self, _username: &str) -> Result<()> {
        let req = PluginRequestC {
            message_id: VarInt(0),
            channel: Bounded("velocity:player_info"),
            data: Bounded(Bytes(&[3])),
        };

        self.0.io.tx(&req).await?;

        Ok(())
    }

    async fn begin_play(&self) -> Result<()> {
        let mut packet_state = self.0.packet_state.write().await;
        *packet_state = PacketState::Play;

        let state = self.0.crawlstate.clone();

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
            dimension_type: state.registry_cache.the_end_id,
            dimension_name: Bounded::<&'static str>("minecraft:the_end"),
            hashed_seed: 0,
            gamemode: Gamemode::Creative,
            previous_gamemode: Some(Gamemode::Adventure),
            is_debug: false,
            is_superflat: false,
            death_location: None,
            portal_cooldown: VarInt(0),
            enforces_secure_chat: false,
        };

        self.0.io.tx(&login).await?;

        self.0
            .io
            .tx(&SetTickingStateC {
                tick_rate: 20.0,
                is_frozen: false,
            })
            .await?;

        self.0.io.tx(&StepTicksC(10)).await?;

        let spawnpoint = state.spawnpoint;
        self.teleport_awaiting(spawnpoint.0, spawnpoint.1, spawnpoint.2, 0.0, 0.0)
            .await?;

        self.0
            .io
            .tx(&SetBorderCenterC {
                x: spawnpoint.0,
                z: spawnpoint.2,
            })
            .await?;

        self.0
            .io
            .tx(&SetBorderSizeC(state.border_radius as f64 * 2.0))
            .await?;

        self.0
            .io
            .tx(&PlayerInfoUpdateC {
                players: &[PlayerStatus::for_player(self.uuid().await)
                    .add_player("You're alone...", &[])
                    .update_listed(true)],
            })
            .await?;

        let await_chunks = GameEventC::from(GameEvent::StartWaitingForLevelChunks);
        self.0.io.tx(&await_chunks).await?;

        let set_center = SetCenterChunkC {
            x: VarInt(spawnpoint.0.floor() as i32 / 16),
            y: VarInt(spawnpoint.2.floor() as i32 / 16),
        };
        self.0.io.tx(&set_center).await?;

        // FIXME: GROSS LOL?????? this should(?) change ownership of the player to the server
        // thread but realistically who knows burhhhh
        state.player_send.send(self.clone()).await?;
        self.spawn_read_loop();

        Ok(())
    }

    pub async fn handle_all_packets(&self) -> Result<()> {
        let packets = {
            let mut frame_queue = self.0.frame_queue.lock().await;
            std::mem::take(&mut *frame_queue)
        };

        for packet in packets {
            self.handle_frame(packet).await?;
        }

        Ok(())
    }

    pub async fn keepalive(&self) -> Result<()> {
        let last_keepalive = self.0.last_keepalive.read().await;
        let now = Instant::now();

        if now - *last_keepalive < Duration::from_secs(10) {
            return Ok(());
        }

        drop(last_keepalive);
        let mut last_keepalive = self.0.last_keepalive.write().await;
        *last_keepalive = now;

        let id = {
            let mut rng = rand::thread_rng();
            rng.gen()
        };

        // if this times out then the player just hasn't requested ping yet lol
        match timeout(Duration::from_secs(1), self.ping(id)).await {
            Ok(Ok(())) | Err(_) => Ok(()),
            Ok(Err(why)) => Err(why),
        }
    }

    async fn ping(&self, id: i64) -> Result<()> {
        let ka = KeepAliveC(id);
        self.0.io.tx(&ka).await?;
        // TODO: check return keepalive, kick
        Ok(())
    }

    pub async fn uuid(&self) -> Uuid {
        let uuid = self.0.uuid.read().await;
        uuid.expect("uuid() called on uninitialized player - only call this after login!")
    }

    pub async fn teleport_awaiting(
        &self,
        x: f64,
        y: f64,
        z: f64,
        yaw: f32,
        pitch: f32,
    ) -> Result<()> {
        {
            let tp_state = self.0.tp_state.read().await;
            if *tp_state != TeleportState::Clear {
                bail!("Player {} already has a teleport pending", self.0.id);
            };
        }

        let tp = SynchronisePositionC::new(x, y, z, yaw, pitch);
        {
            let mut tp_state = self.0.tp_state.write().await;
            // player will be given 5 (FIVE) SECONDS TO ACK!!!!!
            *tp_state = TeleportState::Pending(tp.id, Instant::now());
        }
        self.0.io.tx(&tp).await?;

        let tp_ack = self.0.io.rx::<ConfirmTeleportS>().await?;
        let tp_ack = tp_ack.decode::<ConfirmTeleportS>()?;

        match tokio::time::timeout(Duration::from_secs(5), self.confirm_teleport(tp_ack.id)).await {
            Ok(Ok(())) => {
                let mut tp_state = self.0.tp_state.write().await;
                *tp_state = TeleportState::Clear;
            }
            Ok(Err(why)) => {
                warn!("Spawning player {} failed: {why}", self.0.id);
                Err(why)?;
            }
            Err(why) => {
                warn!("Spawning player {} timed out: {why}", self.0.id);
                Err(why)?;
            }
        }
        Ok(())
    }

    async fn confirm_teleport(&self, id: i32) -> Result<(), TeleportError> {
        let tp_state = self.0.tp_state.read().await;
        match *tp_state {
            TeleportState::Clear => Err(TeleportError::Unexpected),
            TeleportState::Pending(expected, _) if id == expected => Ok(()),
            TeleportState::Pending(expected, _) => Err(TeleportError::WrongId(expected, id)),
        }
    }

    pub async fn check_teleports(
        &self,
        ack: Option<ConfirmTeleportS>,
    ) -> Result<(), TeleportError> {
        let tp_state = self.0.tp_state.read().await;

        match *tp_state {
            TeleportState::Pending(pending_id, sent_at) => {
                if Instant::now() - sent_at > Duration::from_secs(5) {
                    return Err(TeleportError::TimedOut);
                }

                match ack {
                    Some(ack) if ack.id == pending_id => {
                        drop(tp_state);
                        let mut tp_state = self.0.tp_state.write().await;
                        *tp_state = TeleportState::Clear;
                        Ok(())
                    }
                    Some(ack) => Err(TeleportError::WrongId(ack.id, pending_id)),
                    None => Err(TeleportError::Pending(pending_id)),
                }
            }
            TeleportState::Clear => match ack {
                None => Ok(()),
                Some(_) => Err(TeleportError::Unexpected),
            },
        }
    }

    fn spawn_read_loop(&self) {
        let player = self.clone();

        tokio::spawn(async move {
            loop {
                match player.0.io.rx_raw().await {
                    Ok(frame) => {
                        let mut queue = player.0.frame_queue.lock().await;
                        queue.push(frame);
                    }
                    Err(why) => {
                        if let Some(tokio::io::ErrorKind::UnexpectedEof) =
                            why.downcast_ref::<tokio::io::Error>().map(|e| e.kind())
                        {
                            return;
                        }
                    }
                }
            }
        });
    }

    async fn handle_frame(&self, frame: Frame) -> Result<()> {
        match frame.id {
            SetPlayerPositionS::ID => {
                let packet: SetPlayerPositionS = frame.decode()?;

                let tp_state = self.0.tp_state.read().await;
                if *tp_state == TeleportState::Clear {
                    let mut entity = self.0.entity.write().await;
                    entity.reposition(packet.x, packet.feet_y, packet.z);
                }
            }

            SetPlayerPositionAndRotationS::ID => {
                let packet: SetPlayerPositionAndRotationS = frame.decode()?;

                let tp_state = self.0.tp_state.read().await;
                if *tp_state == TeleportState::Clear {
                    let mut entity = self.0.entity.write().await;
                    entity.reposition(packet.x, packet.feet_y, packet.z);
                    entity.rotate(packet.yaw, packet.pitch);
                }
            }

            ConfirmTeleportS::ID => {
                let packet: ConfirmTeleportS = frame.decode()?;
                self.check_teleports(Some(packet)).await?;
            }

            UseItemOnS::ID => {
                let packet: UseItemOnS = frame.decode()?;
                self.handle_use_item(packet).await?;
            }

            id => {
                debug!(
                    "Got packet with id {id} from player {}, ignoring",
                    self.0.id
                );
            }
        }

        Ok(())
    }

    async fn handle_use_item(&self, packet: UseItemOnS) -> Result<()> {
        let id = self.0.next_window_id.fetch_add(1, Ordering::Relaxed);

        let window = Window {
            id,
            kind: WindowType::Generic9x3,
            title: "Hi".into(),
        };

        self.0.io.tx(&OpenScreenC::from(&window)).await?;

        {
            let mut sw = self.0.window.write().await;
            *sw = Some(window);
        }

        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum TeleportError {
    #[error("Client was not expecting a teleport acknowledgement")]
    Unexpected,
    #[error("Teleport ack has wrong ID (expected {0}, got {1})")]
    WrongId(i32, i32),
    #[error("Teleport timed out")]
    TimedOut,
    #[error("Waiting for teleport acknowledgement for id {0}")]
    Pending(i32),
}
