use crawlspace_macro::{Packet, Read};
use crawlspace_proto::{ConnectionState, PacketId::Numeric};

#[derive(Packet, Read)]
#[packet(
    id = "Numeric(0x00)",
    state = "ConnectionState::Handshake",
    serverbound
)]
pub struct HandshakeS {}
