use flate2::{write::ZlibEncoder, Compression};

use crate::WriteExt;
use std::io::{Result, Write};

macro_rules! builder_write_type {
    ($name:tt, $type:ty) => {
        pub fn $name(&mut self, value: $type) -> Result<()> {
            self.buffer.$name(value)
        }
    };
}

#[derive(Debug)]
pub struct PacketBuilder {
    _id: i32,
    buffer: Vec<u8>,
}

impl PacketBuilder {
    pub fn new(id: i32) -> Result<PacketBuilder> {
        let mut buffer = Vec::new();
        buffer.write_varint(id)?;

        Ok(PacketBuilder { _id: id, buffer })
    }

    builder_write_type!(write_boolean, bool);
    builder_write_type!(write_byte, i8);
    builder_write_type!(write_ubyte, u8);
    builder_write_type!(write_short, i16);
    builder_write_type!(write_ushort, u16);
    builder_write_type!(write_int, i32);
    builder_write_type!(write_long, i64);
    builder_write_type!(write_float, f32);
    builder_write_type!(write_double, f64);
    builder_write_type!(write_varint, i32);

    builder_write_type!(write_byte_array, &[u8]);

    pub fn write_string<S: Into<String>>(&mut self, value: S) -> Result<()> {
        self.buffer.write_string(value)
    }

    pub fn write_to<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer.write_varint(self.buffer.len() as i32)?;
        writer.write_all(&self.buffer)
    }

    pub fn write_compressed<W: Write>(&self, writer: &mut W, threshold: usize) -> Result<()> {
        let mut compressed_buffer = Vec::new();

        let compressed_buffer = if self.buffer.len() >= threshold {
            compressed_buffer.write_varint(self.buffer.len() as i32)?;

            let mut encoder = ZlibEncoder::new(compressed_buffer, Compression::default());
            encoder.write_all(&self.buffer)?;

            encoder.finish()?
        } else {
            compressed_buffer.write_varint(0)?;
            compressed_buffer.write_all(&self.buffer)?;
            compressed_buffer
        };

        writer.write_varint(compressed_buffer.len() as i32)?;
        writer.write_all(&compressed_buffer)
    }
}
