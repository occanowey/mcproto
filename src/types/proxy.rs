use std::io::Read;

use crate::{
    error::Result,
    types::{v32, McRead, McWrite},
    PacketBuilder,
};

pub mod i32_as_v32 {
    use super::*;

    pub fn read<R: Read>(reader: &mut R, _remaining_length: usize) -> Result<(i32, usize)> {
        let (value, value_length) = v32::read(reader)?;
        Ok((value.0, value_length))
    }

    pub fn write(packet: &mut PacketBuilder, value: &i32) -> Result<()> {
        Ok(packet.write(&v32(*value))?)
    }
}

// TODO: merge both write functions... somehow
// and also maybe reads but they're not so bad

pub mod bool_option {
    use super::*;

    pub fn read<R: Read, T: McRead>(
        reader: &mut R,
        _remaining_length: usize,
    ) -> Result<(Option<T>, usize)> {
        Ok(self::mc_read(reader)?)
    }

    pub fn write<T: McWrite>(packet: &mut PacketBuilder, value: &Option<T>) -> Result<()> {
        packet.write(&value.is_some())?;

        if let Some(value) = value {
            packet.write(value)?;
        }

        Ok(())
    }

    pub fn mc_read<R: std::io::prelude::Read, T: McRead>(
        reader: &mut R,
    ) -> std::io::Result<(Option<T>, usize)> {
        let (has_value, mut total_value_len) = bool::read(reader)?;

        let value = if has_value {
            let (value, value_len) = T::read(reader)?;
            total_value_len += value_len;
            Some(value)
        } else {
            None
        };

        Ok((value, total_value_len))
    }

    pub fn mc_write<W: std::io::prelude::Write, T: McWrite>(
        writer: &mut W,
        value: &Option<T>,
    ) -> std::io::Result<()> {
        value.is_some().write(writer)?;

        if let Some(value) = value {
            value.write(writer)?;
        }

        Ok(())
    }
}

pub mod length_prefix_bytes {
    use super::*;

    pub fn read<R: Read>(reader: &mut R, _remaining_length: usize) -> Result<(Vec<u8>, usize)> {
        Ok(self::mc_read(reader)?)
    }

    pub fn write<B: AsRef<[u8]>>(packet: &mut PacketBuilder, value: B) -> Result<()> {
        let bytes = value.as_ref();

        i32_as_v32::write(packet, &(bytes.len() as _))?;
        Ok(packet.write_byte_array(bytes)?)
    }

    pub fn mc_read<R: Read>(reader: &mut R) -> std::io::Result<(Vec<u8>, usize)> {
        let (buffer_len, len_len) = v32::read(reader)?;

        let mut buffer = vec![0; *buffer_len as usize];
        reader.read_exact(&mut buffer)?;

        Ok((buffer, *buffer_len as usize + len_len))
    }
}

pub mod length_prefix_array {
    use super::*;

    pub fn read<R: Read, T: McRead>(
        reader: &mut R,
        _remaining_length: usize,
    ) -> Result<(Vec<T>, usize)> {
        Ok(self::mc_read(reader)?)
    }

    pub fn write<T: McWrite, A: AsRef<[T]>>(packet: &mut PacketBuilder, value: A) -> Result<()> {
        let array = value.as_ref();

        let values_count = array.len();
        // TODO: return error rather than panic
        assert!(values_count < (v32::MAX as usize));
        i32_as_v32::write(packet, &(values_count as _))?;

        for value in array {
            packet.write(value)?;
        }

        Ok(())
    }

    pub fn mc_read<R: Read, T: McRead>(reader: &mut R) -> std::io::Result<(Vec<T>, usize)> {
        let mut values = Vec::new();

        let (values_count, mut length) = v32::read(reader)?;
        for _ in 0..values_count.0 {
            let (value, value_length) = T::read(reader)?;

            values.push(value);
            length += value_length;
        }

        Ok((values, length))
    }

    pub fn mc_write<W: std::io::prelude::Write, T: McWrite, A: AsRef<[T]>>(
        writer: &mut W,
        value: A,
    ) -> std::io::Result<()> {
        let array = value.as_ref();

        let values_count = array.len();
        // TODO: return error rather than panic
        assert!(values_count < (v32::MAX as usize));
        v32(values_count as _).write(writer)?;

        for value in array {
            value.write(writer)?;
        }

        Ok(())
    }
}

pub mod remaining_bytes {
    use super::*;

    pub fn read<R: Read>(reader: &mut R, remaining_length: usize) -> Result<(Vec<u8>, usize)> {
        let mut buffer = vec![0; remaining_length];
        reader.read_exact(&mut buffer)?;

        Ok((buffer, remaining_length))
    }

    pub fn write<B: AsRef<[u8]>>(packet: &mut PacketBuilder, value: B) -> Result<()> {
        let bytes = value.as_ref();
        Ok(packet.write_byte_array(bytes)?)
    }
}
