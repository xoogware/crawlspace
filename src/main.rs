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
use server::Server;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[macro_use]
extern crate tracing;

mod net;
mod protocol;
mod server;
mod state;

const VERSION: &str = "1.21.1";
const VERSION_NUM: i32 = 767;
const DESCRIPTION: &str = "sheldon cooper residence";
const MAX_PLAYERS: usize = 906;
const TICK_RATE: u8 = 20;

type CrawlState = Arc<state::State>;

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

    let state = Arc::new(state::State::new(
        VERSION,
        VERSION_NUM,
        DESCRIPTION,
        MAX_PLAYERS,
    ));

    let port = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse::<u16>().ok())
        .unwrap_or(25565);

    net::spawn_net_handler(state.clone(), port).await?;

    let server = Server::new(state.clone(), TICK_RATE);

    {
        let mut ticker = server.ticker;
        tokio::spawn(async move { ticker.run(server).await });
    }

    // TODO: more graceful shutdown?
    tokio::signal::ctrl_c().await?;

    Ok(())
}
