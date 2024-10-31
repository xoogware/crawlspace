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

use std::io::Write;

use color_eyre::eyre::Result;
use serde::Serialize;

use crate::protocol::{Decode, Encode, Packet};

#[derive(Debug)]
pub struct StatusRequestS;

impl Packet for StatusRequestS {
    const ID: i32 = 0x00;
}

impl<'a> Decode<'a> for StatusRequestS {
    fn decode(_buf: &mut &'a [u8]) -> Result<Self> {
        Ok(Self)
    }
}

#[derive(Debug)]
pub struct StatusResponseC<'a> {
    json_respose: StatusJSON<'a>,
}

#[derive(Debug, Serialize)]
struct StatusJSON<'a> {
    version: Version<'a>,
    players: Players,
    description: Option<Description<'a>>,
    enforces_secure_chat: bool,
}

#[derive(Debug, Serialize)]
struct Version<'a> {
    name: &'a str,
}

#[derive(Debug, Serialize)]
struct Players {
    max: i32,
    online: i32,
}

#[derive(Debug, Serialize)]
struct Description<'a> {
    text: &'a str,
}

impl<'a> StatusResponseC<'a> {
    pub fn new(
        version_name: &'a str,
        online_players: i32,
        max_players: i32,
        description: Option<&'a str>,
    ) -> Self {
        Self {
            json_respose: StatusJSON {
                version: Version { name: version_name },
                players: Players {
                    online: online_players,
                    max: max_players,
                },
                description: description.map(|t| Description { text: t }),
                enforces_secure_chat: false,
            },
        }
    }
}

impl Encode for StatusResponseC {
    fn encode(&self, w: impl Write) -> Result<()> {
        Ok(())
    }
}
