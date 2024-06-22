use std::io::{Read, Result};

impl<R: Read + ?Sized> VarintReadExt for R {}

pub trait VarintReadExt: Read {
    #[inline]
    fn read_ubyte(&mut self) -> Result<u8> {
        let mut buffer = [0; 1];
        self.read_exact(&mut buffer)?;
        Ok(<u8>::from_be_bytes(buffer))
    }

    #[inline]
    fn read_varint(&mut self) -> Result<(i32, usize)> {
        let mut acc = 0;
        let mut i = 0;

        loop {
            let byte = self.read_ubyte()? as i32;
            acc |= (byte & 0x7F) << (i * 7);

            i += 1;
            if i > 5 {
                // TODO
                panic!("varint too big");
            }

            if (byte & 0b10000000) == 0 {
                break;
            }
        }

        Ok((acc, i))
    }
}
