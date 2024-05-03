use super::{impl_packet_enum, Packet, PacketBuilder, PacketRead, PacketWrite};
use crate::{error::Result, types::McRead};
use packet_derive::{Packet, PacketRead, PacketWrite};
use std::io::Read;

//
// Clientbound
//

impl_packet_enum!(s2c {
    0x1A => Disconnect,
});

#[derive(Debug, Packet, PacketRead, PacketWrite)]
#[packet(id = 0x1A)]
pub struct Disconnect {
    pub reason: String,
}

//
// Serverbound
//

impl_packet_enum!(c2s {});
