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

use parking_lot::Mutex;
use world::World;

use crate::net::player::Player;

use self::ticker::Ticker;

pub struct Server {
    pub ticker: Ticker,

    _world: Option<Arc<World>>,
    players: Arc<Mutex<Vec<Player>>>,
}

impl Server {
    #[must_use]
    pub fn new(tick_rate: u8) -> Self {
        Server {
            ticker: Ticker::new(tick_rate),
            _world: None,
            players: Arc::new(Mutex::new(Vec::new())),
        }
    }

    async fn tick(&self) {
        let players = self.players.lock();
        players.iter().for_each(|p| trace!("Ticking {}", p.id));
    }
}
