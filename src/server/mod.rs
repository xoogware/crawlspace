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

mod world;

use std::sync::Arc;

use world::World;

use crate::{net::player::SharedPlayer, CrawlState};

use self::ticker::Ticker;

#[derive(Debug)]
pub struct Server {
    pub ticker: Ticker,

    world: Option<Arc<World>>,
    players: Vec<SharedPlayer>,

    crawlstate: CrawlState,
}

impl Server {
    #[must_use]
    pub fn new(state: CrawlState, tick_rate: u8) -> Self {
        Server {
            ticker: Ticker::new(tick_rate),
            world: None,
            players: Vec::new(),
            crawlstate: state,
        }
    }

    #[tracing::instrument]
    async fn tick(&mut self) {
        let state = self.crawlstate.clone();
        let mut player_recv = state.player_recv.lock().await;

        while let Ok(p) = player_recv.try_recv() {
            self.players.push(p);
        }
    }
}
