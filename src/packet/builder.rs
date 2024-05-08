use std::io::Write;

use bytes::{BufMut, BytesMut};
use flate2::{write::ZlibEncoder, Compression};

use crate::{
    error::Result,
    types::{proxy::i32_as_v32, BufType},
    varint::VarintWriteExt,
};

#[derive(Debug)]
pub struct PacketBuilder {
    _id: i32,
    buffer: BytesMut,
}

impl PacketBuilder {
    pub fn new(id: i32) -> Result<Self> {
        let mut buffer = BytesMut::new();
        id.buf_write(&mut buffer)?;

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
            i32_as_v32::buf_write(&(self.buffer.len() as _), &mut compressed_buffer)?;

            let mut encoder = ZlibEncoder::new(compressed_buffer.writer(), Compression::default());
            encoder.write_all(&self.buffer)?;

            encoder.finish()?.into_inner()
        } else {
            i32_as_v32::buf_write(&0, &mut compressed_buffer)?;
            compressed_buffer.put(self.buffer.clone());
            compressed_buffer
        };

        writer.write_varint(compressed_buffer.len() as _)?;
        Ok(writer.write_all(&compressed_buffer)?)
    }
}
