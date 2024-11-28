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

use std::sync::{atomic::AtomicUsize, Arc};

use tokio::sync::{mpsc, Mutex, RwLock, Semaphore};
use tokio_util::sync::CancellationToken;

use crate::{
    args::Args,
    net::{cache::RegistryCache, player::SharedPlayer},
    protocol::packets::login::registry::ALL_REGISTRIES,
    server::Server,
};

#[derive(Debug)]
pub struct State {
    pub max_players: usize,
    pub current_players: AtomicUsize,
    pub description: String,
    pub version_name: String,
    pub version_number: i32,
    pub addr: String,
    pub port: u16,

    pub registry_cache: RegistryCache,

    pub player_send: mpsc::Sender<SharedPlayer>,
    pub player_recv: Mutex<mpsc::Receiver<SharedPlayer>>,

    pub shutdown_token: CancellationToken,

    pub net_sema: Arc<Semaphore>,

    pub spawnpoint: (f64, f64, f64),
    pub border_radius: i32,

    server: RwLock<Option<Arc<Server>>>,
}

impl State {
    #[must_use]
    pub fn new(version_name: &str, version_number: i32, args: Args) -> Self {
        let max = args.max_players.min(Semaphore::MAX_PERMITS);

        if max < args.max_players {
            warn!("Requested max player count {} is less than max semaphore permits {max} - limited to {max}.", args.max_players);
        }

        let (player_send, player_recv) = mpsc::channel(16);
        let shutdown_token = CancellationToken::new();

        Self {
            max_players: max,
            current_players: AtomicUsize::new(0),
            description: args.motd,
            version_name: version_name.to_owned(),
            version_number: version_number.to_owned(),
            addr: args.addr,
            port: args.port,

            registry_cache: RegistryCache::from(&*ALL_REGISTRIES),

            player_send,
            player_recv: Mutex::new(player_recv),

            shutdown_token,

            net_sema: Arc::new(Semaphore::new(max)),

            spawnpoint: (args.spawn_x, args.spawn_y, args.spawn_z),
            border_radius: args.border_radius,

            server: RwLock::new(None),
        }
    }

    pub async fn set_server(&self, server: Arc<Server>) {
        let mut write = self.server.write().await;
        *write = Some(server);
    }

    pub async fn get_server(&self) -> Arc<Server> {
        let server = self.server.read().await;
        server
            .clone()
            .expect("state.get_server called before server initialized")
    }
}
