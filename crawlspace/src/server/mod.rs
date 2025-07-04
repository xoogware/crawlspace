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

pub mod registries;
pub mod ticker;
pub mod window;

use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
    time::Duration,
};

use color_eyre::eyre::Result;

use tokio::sync::Mutex;

#[cfg(feature = "timings")]
use tokio::time::Instant;

use crate::{
    net::{
        cache::WorldCache,
        player::{SharedPlayer, TeleportError},
    },
    world::{Container, World},
    CrawlState,
};

use self::ticker::Ticker;

#[derive(Debug)]
pub struct Server {
    pub ticker: Ticker,

    world_cache: Arc<WorldCache>,
    players: Mutex<HashMap<u16, SharedPlayer>>,

    crawlstate: CrawlState,
}

impl Server {
    #[must_use]
    pub fn new(state: CrawlState, world_cache: WorldCache, tick_rate: u8) -> Arc<Self> {
        let server = Arc::new(Server {
            ticker: Ticker::new(tick_rate),
            world_cache: Arc::new(world_cache),
            players: Mutex::new(HashMap::new()),
            crawlstate: state.clone(),
        });

        let state_server = server.clone();
        tokio::spawn(async move {
            state.set_server(state_server).await;
        });

        server
    }

    async fn tick(&self) {
        #[cfg(feature = "timings")]
        let run_start = Instant::now();

        let state = self.crawlstate.clone();
        let mut player_recv = state.player_recv.lock().await;

        let mut players = self.players.lock().await;
        while let Ok(p) = player_recv.try_recv() {
            players.insert(p.0.id, p.clone());
            tokio::spawn(Self::send_world_to(p.clone(), self.world_cache.clone()));
        }

        let mut invalid_players: HashSet<u16> = HashSet::new();

        for (id, player) in &*players {
            let _ = player.keepalive().await;

            match player.handle_all_packets().await {
                Ok(()) => (),
                Err(why) => {
                    error!(
                        "error handling packets for player {}: {:?}",
                        player.id(),
                        why
                    );
                    invalid_players.insert(*id);
                    continue;
                }
            }

            {
                if !player.0.io.connected().await {
                    invalid_players.insert(*id);
                }
            }

            match player.check_teleports(None).await {
                Err(TeleportError::TimedOut) | Err(TeleportError::WrongId(..)) => {
                    warn!("Player {} teleport failed, removing", player.0.id);
                    invalid_players.insert(*id);
                }
                _ => (),
            }
        }

        for id in invalid_players {
            // TODO: kick player properly
            players.remove(&id);
        }

        #[cfg(feature = "timings")]
        {
            let run_end = Instant::now();
            debug!("Tick took {}ms", (run_end - run_start).as_millis());
        }
    }

    async fn send_world_to(player: SharedPlayer, world_cache: Arc<WorldCache>) -> Result<()> {
        for packet in world_cache.encoded.iter() {
            player.0.io.tx_raw(packet).await?;
        }

        Ok(())
    }

    pub fn get_container(&self, x: i32, y: i32, z: i32) -> Option<Container> {
        self.world_cache.containers.get(&(x, y, z)).cloned()
    }
}
