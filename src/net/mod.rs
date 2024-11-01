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

use color_eyre::eyre::Result;
use player::Player;
use tokio::net::TcpListener;
use uuid::Uuid;

mod io;
mod player;

use crate::CrawlState;

pub async fn spawn_net_handler(state: CrawlState, port: u16) -> Result<()> {
    let listener = TcpListener::bind(format!("0.0.0.0:{port}")).await?;
    warn!("Listening on port {port}.");
    tokio::spawn(net_handler(state, listener));
    Ok(())
}

async fn net_handler(crawlstate: CrawlState, listener: TcpListener) {
    let state = crawlstate.clone();
    let mut client_counter: u16 = 0;
    loop {
        // it feels weird to clone on each loop here but that's what it says to do so i guess it's
        // fine?
        match state.net_sema.clone().acquire_owned().await {
            Err(_) => {
                warn!("Net semaphore closed, stopping listener!");
                return;
            }
            Ok(permit) => match listener.accept().await {
                Err(why) => error!("Failed to accept client, dropping: {why}"),
                Ok((conn, addy)) => {
                    let client_id = client_counter;
                    client_counter = client_counter.wrapping_add(1);
                    let state = crawlstate.clone();

                    tokio::spawn(async move {
                        // TODO: handle initial connection here! DO NOT DROP PERMIT UNTIL PLAYER
                        // DISCONNECTS
                        tokio::spawn(async move {
                            let mut player = Player::new(state, client_id, conn);
                            player.connect().await;
                        });

                        // moves permit to this closure so it must be dropped by the spawned task
                        std::mem::drop(permit);
                    });
                }
            },
        }
    }
}
