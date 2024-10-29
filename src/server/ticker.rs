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

use tokio::time::{sleep, Instant};

#[derive(Clone, Copy)]
pub struct Ticker {
    tick_interval: Duration,
    last_tick: Instant,
}

impl Ticker {
    #[must_use]
    pub fn new(tick_rate: u8) -> Self {
        Self {
            tick_interval: Duration::from_millis((1000.0 / tick_rate as f64) as u64),
            last_tick: Instant::now(),
        }
    }

    pub async fn run(&mut self, server: &super::Server) {
        loop {
            let now = Instant::now();
            let elapsed = now - self.last_tick;

            if elapsed <= self.tick_interval {
                sleep(self.tick_interval - elapsed).await;
                continue;
            }

            self.last_tick = now;
            trace!("{}ms elapsed, ticking full server", elapsed.as_millis(),);
            server.tick().await;
        }
    }
}
