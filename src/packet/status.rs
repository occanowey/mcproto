use super::{Packet, PacketBuilder, PacketRead, PacketWrite};
use crate::types::McRead;
use packet_derive::{Packet, PacketRead, PacketWrite};
use std::io::{Read, Result};

//
// Clientbound
//

#[derive(Debug, Packet, PacketRead, PacketWrite)]
#[id(0x00)]
pub struct Response {
    pub response: String,
}

#[derive(Debug, Packet, PacketRead, PacketWrite)]
#[id(0x01)]
pub struct Pong {
    pub data: i64,
}

//
// Serverbound
//

#[derive(Debug, Packet, PacketRead, PacketWrite)]
#[id(0x00)]
pub struct Request;

#[derive(Debug, Packet, PacketRead, PacketWrite)]
#[id(0x01)]
pub struct Ping {
    pub data: i64,
}
