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

use tokio::{io::AsyncReadExt, net::TcpStream};
use uuid::Uuid;

pub struct Player {
    pub id: u16,
    stream: TcpStream,
    state: ConnectionState,
}

enum ConnectionState {
    Connecting,
    Playing(Uuid),
    Broken,
    Closed,
}

impl Player {
    #[must_use]
    pub fn new(id: u16, connection: TcpStream) -> Self {
        Self {
            id,
            stream: connection,
            state: ConnectionState::Connecting,
        }
    }

    pub async fn run(&mut self) {
        loop {
            match self.state {
                ConnectionState::Connecting => self.handle_connecting().await,
                ConnectionState::Playing(uuid) => self.play(uuid).await,
                ConnectionState::Closed | ConnectionState::Broken => {
                    info!("Player {} closed.", self.id);
                    break;
                }
            }
        }
    }

    async fn handle_connecting(&mut self) {}

    async fn play(&mut self, uuid: Uuid) {}

    async fn read(&mut self) {}
}
