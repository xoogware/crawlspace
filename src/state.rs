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

use tokio::sync::Semaphore;

pub struct State {
    pub max_players: usize,
    pub current_players: AtomicUsize,
    pub description: String,
    pub version_name: String,
    pub version_number: i32,

    pub net_sema: Arc<Semaphore>,
}

impl State {
    #[must_use]
    pub fn new(
        version_name: &str,
        version_number: i32,
        description: &str,
        max_players: usize,
    ) -> Self {
        let max = max_players.min(Semaphore::MAX_PERMITS);

        if max < max_players {
            warn!("Requested max player count {max_players} is less than max semaphore permits {max} - limited to {max}.");
        }

        Self {
            max_players: max,
            current_players: AtomicUsize::new(0),
            description: description.to_owned(),
            version_name: version_name.to_owned(),
            version_number: version_number.to_owned(),
            net_sema: Arc::new(Semaphore::new(max)),
        }
    }
}
