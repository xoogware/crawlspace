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
    pub map_dir: String,
    /// The address to serve crawlspace on. Defaults to [::] (all interfaces) if not set.
    #[arg(short, long)]
    addr: Option<String>,
    /// The port to serve crawlspace on. Defaults to 25565 if not set.
    #[arg(short, long)]
    port: Option<u16>,
}

impl Args {
    #[inline(always)]
    pub fn addr(&self) -> String {
        self.addr.clone().unwrap_or("[::]".into())
    }

    #[inline(always)]
    pub fn port(&self) -> u16 {
        self.port.unwrap_or(25565)
    }
}
