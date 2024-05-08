use std::io::{Read, Result, Write};

impl<R: Read + ?Sized> VarintReadExt for R {}
impl<W: Write + ?Sized> VarintWriteExt for W {}

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

pub trait VarintWriteExt: Write {
    fn write_ubyte(&mut self, value: u8) -> Result<()> {
        self.write_all(&value.to_be_bytes())
    }

    fn write_varint(&mut self, value: i32) -> Result<()> {
        let mut input = value as u32;

        loop {
            if (input & 0xFFFFFF80) == 0 {
                break;
            }

            self.write_ubyte((input & 0x7F | 0x80) as u8)?;
            input >>= 7;
        }

        self.write_ubyte((input & 0xFF) as u8)
    }
}
