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

use tokio::sync::{mpsc, Mutex, Semaphore};

use crate::net::player::Player;

#[derive(Debug)]
pub struct State {
    pub max_players: usize,
    pub current_players: AtomicUsize,
    pub description: String,
    pub version_name: String,
    pub version_number: i32,
    pub port: u16,

    pub player_send: mpsc::Sender<Arc<Player>>,
    pub player_recv: Mutex<mpsc::Receiver<Arc<Player>>>,

    pub shutdown_send: mpsc::UnboundedSender<()>,
    pub shutdown_recv: Mutex<mpsc::UnboundedReceiver<()>>,

    pub net_sema: Arc<Semaphore>,
}

impl State {
    #[must_use]
    pub fn new(
        version_name: &str,
        version_number: i32,
        description: &str,
        max_players: usize,
        port: u16,
    ) -> Self {
        let max = max_players.min(Semaphore::MAX_PERMITS);

        if max < max_players {
            warn!("Requested max player count {max_players} is less than max semaphore permits {max} - limited to {max}.");
        }

        let (player_send, player_recv) = mpsc::channel(16);
        let (shutdown_send, shutdown_recv) = mpsc::unbounded_channel();

        Self {
            max_players: max,
            current_players: AtomicUsize::new(0),
            description: description.to_owned(),
            version_name: version_name.to_owned(),
            version_number: version_number.to_owned(),
            port,

            player_send,
            player_recv: Mutex::new(player_recv),

            shutdown_send,
            shutdown_recv: Mutex::new(shutdown_recv),

            net_sema: Arc::new(Semaphore::new(max)),
        }
    }
}
