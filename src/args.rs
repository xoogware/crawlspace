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

use clap::Parser;

#[derive(Debug, Parser)]
pub struct Args {
    /// The directory to load the map from. Should be DIM1, or the equivalent renamed folder.
    #[arg(env = "LIMBO_WORLD")]
    pub map_dir: String,
    /// The address to serve crawlspace on.
    #[arg(short, long, default_value = "[::]", env = "LIMBO_ADDRESS")]
    pub addr: String,
    /// The port to serve crawlspace on. Defaults to 25565 if not set.
    #[arg(short, long, default_value = "25565", env = "LIMBO_PORT")]
    pub port: u16,
    /// The x coordinate of the spawnpoint.
    #[arg(short = 'x', long, default_value = "0", env = "LIMBO_SPAWN_X")]
    pub spawn_x: f64,
    /// The y coordinate of the spawnpoint.
    #[arg(short = 'y', long, default_value = "100", env = "LIMBO_SPAWN_Y")]
    pub spawn_y: f64,
    /// The z coordinate of the spawnpoint.
    #[arg(short = 'z', long, default_value = "0", env = "LIMBO_SPAWN_Z")]
    pub spawn_z: f64,
    /// The border radius, centered around the spawnpoint. Defaults to 10 chunks. One
    /// chunk past the border will be loaded.
    #[arg(short = 'b', long, default_value = "160", env = "LIMBO_BORDER_RADIUS")]
    pub border_radius: i32,
    #[arg(short, long, default_value = "Limbo")]
    pub motd: String,
    #[arg(long, default_value = "500", env = "LIMBO_MAX_PLAYERS")]
    pub max_players: usize,
}
