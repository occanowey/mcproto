mod builder;

pub mod handshaking;
pub mod login;
pub mod status;

use std::io::{Error, ErrorKind, Read, Result, Write};

use crate::ReadExt;

pub use builder::PacketBuilder;

pub trait Packet {
    const PACKET_ID: i32;
}

pub trait PacketRead: Packet + Sized {
    fn read<R: Read>(reader: &mut R) -> Result<Self> {
        let (length, _) = reader.read_varint()?;

        let (id, id_len) = reader.read_varint()?;
        if id != Self::PACKET_ID {
            return Err(Error::new(ErrorKind::Other, "Invalid packet id"));
        }

        Self::read_data(reader, length as usize - id_len)
    }

    /// Read fields after length & id
    fn read_data<R: Read>(reader: &mut R, data_length: usize) -> Result<Self>;
}

pub trait PacketWrite: Packet {
    fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        let mut packet = PacketBuilder::new(Self::PACKET_ID)?;
        self.write_data(&mut packet)?;
        packet.write_to(writer)
    }

    fn write_data(&self, packet: &mut PacketBuilder) -> Result<()>;
}
