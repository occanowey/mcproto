use super::{impl_packet_enum, Packet, PacketBuilder, PacketRead, PacketWrite};
use crate::types::McRead;
use packet_derive::{Packet, PacketRead, PacketWrite};
use std::io::{Read, Result};

//
// Clientbound
//

impl_packet_enum!(s2c {
    0x1A => Disconnect,
});

#[derive(Debug, Packet, PacketRead, PacketWrite)]
#[id(0x1A)]
pub struct Disconnect {
    pub reason: String,
}

//
// Serverbound
//
