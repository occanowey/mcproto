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

pub mod bool_option {
    // TODO: merge both write functions... somehow
    // and also maybe reads but they're not so bad

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
