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

pub mod ticker;

use std::sync::Arc;

use color_eyre::eyre::Result;

use crate::{
    net::{cache::WorldCache, player::SharedPlayer},
    CrawlState,
};

use self::ticker::Ticker;

#[derive(Debug)]
pub struct Server {
    pub ticker: Ticker,

    world_cache: Arc<WorldCache>,
    players: Vec<SharedPlayer>,

    crawlstate: CrawlState,
}

impl Server {
    #[must_use]
    pub fn new(state: CrawlState, world_cache: WorldCache, tick_rate: u8) -> Self {
        Server {
            ticker: Ticker::new(tick_rate),
            world_cache: Arc::new(world_cache),
            players: Vec::new(),
            crawlstate: state,
        }
    }

    async fn tick(&mut self) {
        let state = self.crawlstate.clone();
        let mut player_recv = state.player_recv.lock().await;

        while let Ok(p) = player_recv.try_recv() {
            self.players.push(p.clone());
            tokio::spawn(Self::send_world_to(p.clone(), self.world_cache.clone()));
        }

        for player in &self.players {
            let _ = player.keepalive().await;
            let _ = player.handle_all_packets().await;
        }
    }

    async fn send_world_to(player: SharedPlayer, world_cache: Arc<WorldCache>) -> Result<()> {
        let mut io = player.0.io.lock().await;

        for packet in world_cache.encoded.iter() {
            io.tx_raw(packet).await?;
        }

        Ok(())
    }
}
