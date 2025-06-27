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

use std::{fs::OpenOptions, sync::Arc};

use args::Args;
use clap::Parser;
use color_eyre::eyre::Result;
use net::cache::{RegistryCache, WorldCache};
use server::Server;
use tracing_subscriber::{layer::SubscriberExt, prelude::*, EnvFilter};
use world::read_world;

#[macro_use]
extern crate tracing;

mod args;
mod net;
mod protocol;
mod server;
mod state;
mod world;

const VERSION: &str = "1.21.4";
const VERSION_NUM: i32 = 769;
const TICK_RATE: u8 = 20;

type CrawlState = Arc<state::State>;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    match cfg!(debug_assertions) {
        true => {
            let filter = EnvFilter::from_default_env();
            let fmt = tracing_subscriber::fmt::layer().pretty();
            let file = OpenOptions::new()
                .write(true)
                .truncate(true)
                .create(true)
                .open("log")
                .unwrap();
            tracing_subscriber::registry()
                .with(filter)
                .with(fmt)
                .with(tracing_subscriber::fmt::layer().with_writer(file))
                .init();
        }
        false => tracing_subscriber::registry()
            .with(tracing_subscriber::fmt::layer())
            .init(),
    }

    let args = Args::parse();

    info!("Loading world");
    let world = read_world(&args.map_dir)?;
    info!("Done.");

    let state = Arc::new(state::State::new(VERSION, VERSION_NUM, args));

    info!("Generating world chunk packets");
    let world_cache = WorldCache::from_anvil(state.clone(), &world);
    info!("Done.");

    #[cfg(feature = "lan")]
    net::spawn_lan_broadcast(state.clone()).await?;

    net::spawn_net_handler(state.clone()).await?;

    let server = Server::new(state.clone(), world_cache, TICK_RATE);

    {
        let mut ticker = server.ticker;
        tokio::spawn(async move { ticker.run(server).await });
    }

    // TODO: more graceful shutdown?
    tokio::signal::ctrl_c().await?;
    state.shutdown_token.cancel();

    Ok(())
}
