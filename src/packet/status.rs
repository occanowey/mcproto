use super::{impl_packet_enum, Packet, PacketBuilder, PacketRead, PacketWrite};
use crate::{error::Result, types::McRead};
use packet_derive::{Packet, PacketRead, PacketWrite};
use std::io::Read;

//
// Clientbound
//

impl_packet_enum!(s2c {
    0x00 => Response,
    0x01 => Pong,
});

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

impl_packet_enum!(c2s {
    0x00 => Request,
    0x01 => Ping,
});

#[derive(Debug, Packet, PacketRead, PacketWrite)]
#[id(0x00)]
pub struct Request;

#[derive(Debug, Packet, PacketRead, PacketWrite)]
#[id(0x01)]
pub struct Ping {
    pub data: i64,
}
