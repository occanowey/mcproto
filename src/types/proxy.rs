use std::io::Read;

use crate::{
    error::Result,
    types::{v32, McRead as _},
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
