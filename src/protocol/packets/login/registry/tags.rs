use std::collections::HashMap;
use crate::protocol::{Encode, Packet};
use crate::protocol::datatypes::{Bounded, VarInt};

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct AllTags(pub HashMap<String, Tags>);

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Tags(pub HashMap<String, Vec<String>>);

impl Packet for AllTags {
    const ID: i32 = 0x0D;
}

impl Encode for AllTags {
    fn encode(&self, mut w: impl std::io::Write) -> color_eyre::Result<()> {
        VarInt(self.0.len() as i32).encode(&mut w)?;

        for (registry, tags) in self.0.clone() {
            Bounded::<&'_ str>(registry.as_str()).encode(&mut w)?;
            tags.encode(&mut w)?;
        }

        Ok(())
    }
}

impl Encode for Tags {
    fn encode(&self, mut w: impl std::io::Write) -> color_eyre::Result<()> {
        VarInt(self.0.len() as i32).encode(&mut w)?;

        for (name, _) in self.0.clone() {
            Bounded::<&'_ str>(name.as_str()).encode(&mut w)?;
            VarInt(0).encode(&mut w)?;
        }

        Ok(())
    }
}