use std::io::{Read, Write};

use crate::{
    error::{Error, Result},
    types::{proxy::i32_as_v32, ReadError},
    varint::{VarintReadExt, VarintWriteExt},
};

use bytes::{Buf, BufMut, Bytes, BytesMut};
use flate2::{write::ZlibEncoder, Compression};

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

                pub fn from_id_data<B: bytes::Buf>(id: i32, data: &mut B) -> std::result::Result<Self, crate::types::ReadError> {
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

        let id = i32_as_v32::buf_read(&mut data)?;
        if id != Self::PACKET_ID {
            return Err(Error::IncorectPacketId(Self::PACKET_ID, id));
        }

        Ok(Self::read_data(&mut data)?)
    }

    /// Read fields after length & id
    fn read_data<B: Buf>(data: &mut B) -> std::result::Result<Self, ReadError>;
}

pub trait PacketWrite: Packet {
    fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        let mut packet = PacketBuilder::new(Self::PACKET_ID)?;
        self.write_data(packet.buf_mut());
        packet.write_to(writer)
    }

    fn write_data<B: BufMut>(&self, buf: &mut B);
}

#[derive(Debug)]
pub struct PacketBuilder {
    _id: i32,
    buffer: BytesMut,
}

impl PacketBuilder {
    pub fn new(id: i32) -> Result<Self> {
        let mut buffer = BytesMut::new();
        i32_as_v32::buf_write(&id, &mut buffer);

        Ok(Self { _id: id, buffer })
    }

    pub fn buf_mut(&mut self) -> &mut BytesMut {
        &mut self.buffer
    }

    pub fn write_to<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer.write_varint(self.buffer.len() as _)?;
        Ok(writer.write_all(&self.buffer)?)
    }

    pub fn write_to_compressed<W: Write>(&self, writer: &mut W, threshold: usize) -> Result<()> {
        let mut compressed_buffer = BytesMut::new();

        let compressed_buffer = if self.buffer.len() >= threshold {
            i32_as_v32::buf_write(&(self.buffer.len() as _), &mut compressed_buffer);

            let mut encoder = ZlibEncoder::new(compressed_buffer.writer(), Compression::default());
            encoder.write_all(&self.buffer)?;

            encoder.finish()?.into_inner()
        } else {
            i32_as_v32::buf_write(&0, &mut compressed_buffer);
            compressed_buffer.put(self.buffer.clone());
            compressed_buffer
        };

        writer.write_varint(compressed_buffer.len() as _)?;
        Ok(writer.write_all(&compressed_buffer)?)
    }
}
