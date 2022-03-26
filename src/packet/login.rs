use super::{Packet, PacketBuilder, PacketRead, PacketWrite};
use crate::types::McRead;
use packet_derive::{Packet, PacketRead, PacketWrite};
use std::io::{Read, Result};

//
// Clientbound
//

#[derive(Debug, Packet, PacketRead, PacketWrite)]
#[id(0x00)]
pub struct Disconnect {
    pub reason: String,
}

//
// Serverbound
//

#[derive(Debug, Packet, PacketRead, PacketWrite)]
#[id(0x00)]
pub struct LoginStart {
    pub username: String,
}
