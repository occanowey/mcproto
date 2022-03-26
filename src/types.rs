use std::{
    io::{Read, Result, Write},
    ops::Deref,
};

pub trait McRead {
    fn read<R: Read>(reader: &mut R) -> Result<(Self, usize)>
    where
        Self: std::marker::Sized;
}

pub trait McWrite {
    fn write<W: Write>(&self, writer: &mut W) -> Result<()>;
}

macro_rules! impl_rprimitive {
    ($self:ty, $length:literal) => {
        impl McRead for $self {
            fn read<R: Read>(reader: &mut R) -> Result<(Self, usize)> {
                let mut buffer = [0; $length];
                reader.read_exact(&mut buffer)?;
                Ok((<$self>::from_be_bytes(buffer), $length))
            }
        }

        impl McWrite for $self {
            fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
                writer.write_all(&self.to_be_bytes())
            }
        }
    };
}

// Boolean
impl McRead for bool {
    fn read<R: Read>(reader: &mut R) -> Result<(Self, usize)> {
        Ok((u8::read(reader)?.0 & 1 == 1, 1))
    }
}

impl McWrite for bool {
    fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        (*self as u8).write(writer)
    }
}

// Byte
impl_rprimitive!(u8, 1);

// Unsigned Byte
impl_rprimitive!(i8, 1);

// Short
impl_rprimitive!(u16, 2);

// Unsigned Short
impl_rprimitive!(i16, 2);

// Int
impl_rprimitive!(i32, 4);

// Long
impl_rprimitive!(i64, 8);

// Float
impl_rprimitive!(f32, 4);

// Double
impl_rprimitive!(f64, 8);

// String
impl McRead for String {
    fn read<R: Read>(reader: &mut R) -> Result<(Self, usize)> {
        let (string_len, len_len) = v32::read(reader)?;
        let mut buffer = vec![0; *string_len as usize];
        reader.read_exact(&mut buffer)?;
        let string = String::from_utf8(buffer).unwrap();

        Ok((string, *string_len as usize + len_len))
    }
}

impl McWrite for String {
    fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        v32(self.len() as i32).write(writer)?;
        writer.write_all(self.as_bytes())
    }
}

// Chat

// Identifier

// VarInt
#[derive(Debug)]
#[allow(non_camel_case_types)]
pub struct v32(pub i32);

impl From<i32> for v32 {
    fn from(inner: i32) -> Self {
        v32(inner)
    }
}

impl Deref for v32 {
    type Target = i32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl McRead for v32 {
    fn read<R: Read>(reader: &mut R) -> Result<(Self, usize)> {
        let mut acc = 0;
        let mut i = 0;

        loop {
            let byte = u8::read(reader)?.0 as i32;
            acc |= (byte & 0x7F) << (i * 7);

            i += 1;
            if i > 5 {
                panic!("varint too big");
            }

            if (byte & 0b10000000) == 0 {
                break;
            }
        }

        Ok((v32(acc), i))
    }
}

impl McWrite for v32 {
    fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        let mut input = self.0 as u32;

        loop {
            if (input & 0xFFFFFF80) == 0 {
                break;
            }

            ((input & 0x7F | 0x80) as u8).write(writer)?;
            input >>= 7;
        }

        (input as u8).write(writer)
    }
}

// VarLong

// Entity Metadata

// Slot

// NBT Tag

// Position

// Angle

// UUID

// Optional X

// Array of X

// X Enum

// Byte Array