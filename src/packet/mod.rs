mod builder;

pub mod handshaking;
pub mod login;
pub mod play;
pub mod status;

use std::io::{Read, Write};

use crate::{
    error::{Error, Result},
    ReadExt,
};

pub use builder::PacketBuilder;

macro_rules! impl_packet_enum {
    ($side:ident {$($id:literal => $packet:ident),* $(,)?}) => {
        pub mod $side {
            use crate::{packet::PacketRead, error::Result};

            #[derive(Debug)]
            pub enum Packet {
                $($packet(super::$packet),)*

                Unknown(i32)
            }

            impl Packet {
                pub fn from_id_data(id: i32, data: &mut &[u8]) -> Result<Self> {
                    match id {
                        $($id => super::$packet::read_data(data, data.len()).map(Self::$packet),)*

                        other => Ok(Self::Unknown(other)),
                    }
                }
            }
        }
    };
}

pub(crate) use impl_packet_enum;

pub trait Packet {
    const PACKET_ID: i32;
}

pub trait PacketRead: Packet + Sized {
    fn read<R: Read>(reader: &mut R) -> Result<Self> {
        let (length, _) = reader.read_varint()?;

        let (id, id_len) = reader.read_varint()?;
        if id != Self::PACKET_ID {
            return Err(Error::IncorectPacketId(Self::PACKET_ID, id));
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
        Ok(packet.write_to(writer)?)
    }

    fn write_data(&self, packet: &mut PacketBuilder) -> Result<()>;
}
