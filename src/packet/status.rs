use super::{impl_packet_enum, Packet, PacketRead, PacketWrite};
use crate::error::Result;
use packet_derive::{Packet, PacketRead, PacketWrite};

//
// Clientbound
//

impl_packet_enum!(s2c {
    0x00 => StatusResponse,
    0x01 => PingResponse,
});

#[derive(Debug, Packet, PacketRead, PacketWrite)]
#[packet(id = 0x00)]
pub struct StatusResponse {
    pub response: String,
}

#[derive(Debug, Packet, PacketRead, PacketWrite)]
#[packet(id = 0x01)]
pub struct PingResponse {
    pub payload: i64,
}

//
// Serverbound
//

impl_packet_enum!(c2s {
    0x00 => StatusRequest,
    0x01 => PingRequest,
});

#[derive(Debug, Packet, PacketRead, PacketWrite)]
#[packet(id = 0x00)]
pub struct StatusRequest;

#[derive(Debug, Packet, PacketRead, PacketWrite)]
#[packet(id = 0x01)]
pub struct PingRequest {
    pub payload: i64,
}
