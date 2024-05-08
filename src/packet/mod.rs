mod builder;

pub mod configuration;
pub mod handshaking;
pub mod login;
pub mod play;
pub mod status;

use std::io::{Read, Write};

use crate::{
    error::{Error, Result},
    types::proxy::i32_as_v32,
    ReadExt,
};

pub use builder::PacketBuilder;

use bytes::{Buf, BufMut, Bytes, BytesMut};

macro_rules! impl_packet_enum {
    ($side:ident {$($id:literal => $packet:ident),* $(,)?}) => {
        pub mod $side {
            #[derive(Debug)]
            pub enum Packet {
                $($packet(super::$packet),)*

                Unknown(i32)
            }

            #[automatically_derived]
            impl Packet {
                pub fn is_known(&self) -> bool {
                    !matches!(self, Self::Unknown(_))
                }

                pub fn from_id_data<B: bytes::Buf>(id: i32, data: &mut B) -> crate::error::Result<Self> {
                    match id {
                        $($id => <super::$packet as crate::packet::PacketRead>::read_data(data).map(Self::$packet),)*

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

        let mut data = vec![0; length as _];
        reader.read_exact(&mut data)?;
        let mut data = Bytes::from(data);

        let (id, _): (i32, usize) = i32_as_v32::buf_read(&mut data)?;
        if id != Self::PACKET_ID {
            return Err(Error::IncorectPacketId(Self::PACKET_ID, id));
        }

        Self::read_data(&mut data)
    }

    /// Read fields after length & id
    fn read_data<B: Buf>(data: &mut B) -> Result<Self>;
}

pub trait PacketWrite: Packet {
    fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        let mut packet = PacketBuilder::new(Self::PACKET_ID)?;
        let mut data = BytesMut::new();
        self.write_data(&mut data)?;
        packet.write_byte_array(&data)?;
        Ok(packet.write_to(writer)?)
    }

    fn write_data<B: BufMut>(&self, buf: &mut B) -> Result<()>;
}
