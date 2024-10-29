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

use std::sync::Arc;

use color_eyre::eyre::Result;
use server::{player::Player, Server};
use tokio::net::TcpListener;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[macro_use]
extern crate tracing;

mod protocol;
mod server;

const TICK_RATE: u8 = 20;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    color_eyre::install()?;

    // RUST_LOG=crawlspace=trace
    match cfg!(debug_assertions) {
        true => {
            let filter = EnvFilter::from_default_env();
            let fmt = tracing_subscriber::fmt::layer().pretty();
            tracing_subscriber::registry().with(filter).with(fmt).init();
        }
        false => tracing_subscriber::fmt::init(),
    }

    let port = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse::<u16>().ok())
        .unwrap_or(25565);

    let listener = TcpListener::bind(format!("0.0.0.0:{port}")).await?;

    let server = Arc::new(Server::new(TICK_RATE));

    {
        let mut ticker = server.clone().ticker;
        tokio::spawn(async move { ticker.run(&server).await });
    }

    warn!("Listening on port {port}.");

    let mut client_counter: u16 = 0;
    loop {
        let (connection, address) = listener.accept().await?;

        let client_id = client_counter;
        client_counter = client_counter.wrapping_add(1);

        info!("New connection (id {client_id}) from {address}");

        if let Err(why) = connection.set_nodelay(true) {
            warn!("Failed to set nodelay for {client_id}: {why}");
        }

        let mut player = Player::new(client_id, connection);
        tokio::spawn(async move {
            player.run().await;
        });
    }
}
