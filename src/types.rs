use std::{
    io::{Read, Result, Write},
    ops::Deref,
};

use uuid::Uuid;

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
impl_rprimitive!(i8, 1);

// Unsigned Byte
impl_rprimitive!(u8, 1);

// Short
impl_rprimitive!(i16, 2);

// Unsigned Short
impl_rprimitive!(u16, 2);

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
        let (buffer, len) = LengthPrefixByteArray::read(reader)?;
        let string = String::from_utf8(buffer.0).unwrap();

        Ok((string, len))
    }
}

impl McWrite for String {
    fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        v32(self.len() as i32).write(writer)?;
        writer.write_all(self.as_bytes())
    }
}

// Text Component
#[derive(Debug)]
pub enum TextComponent {
    String(String),
    Compound(()),
}

impl McRead for TextComponent {
    fn read<R: Read>(reader: &mut R) -> Result<(Self, usize)> {
        todo!()
    }
}

// Chat

// Identifier
// TODO: make sure it's a valid ident?
#[derive(Debug, Hash, PartialEq, Eq)]
pub struct Identifier(pub String);

impl McRead for Identifier {
    fn read<R: Read>(reader: &mut R) -> Result<(Self, usize)> {
        let (data, length) = String::read(reader)?;

        Ok((Identifier(data), length))
    }
}

impl McWrite for Identifier {
    fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        self.0.write(writer)
    }
}

// VarInt
#[derive(Debug)]
#[allow(non_camel_case_types)]
pub struct v32(pub i32);

impl v32 {
    // sourced from https://wiki.vg/VarInt_And_VarLong
    // unsure of accuracy
    pub const MAX: i32 = 2147483647;
}

impl From<i32> for v32 {
    fn from(inner: i32) -> Self {
        v32(inner)
    }
}

impl From<&i32> for v32 {
    fn from(inner: &i32) -> Self {
        v32(*inner)
    }
}

impl From<v32> for i32 {
    fn from(value: v32) -> Self {
        value.0
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
                // TODO: return actual error
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
impl McRead for Uuid {
    fn read<R: Read>(reader: &mut R) -> Result<(Self, usize)> {
        let mut buffer = [0; 16];
        reader.read_exact(&mut buffer)?;

        Ok((Uuid::from_bytes(buffer), buffer.len()))
    }
}

impl McWrite for Uuid {
    fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer.write_all(self.as_bytes())
    }
}

// Optional X

// Array of X

// X Enum

// Byte Array
#[derive(Debug)]
pub struct LengthPrefixByteArray(pub Vec<u8>);

impl<'f> From<&'f [u8]> for LengthPrefixByteArray {
    fn from(bytes: &'f [u8]) -> Self {
        bytes.to_vec().into()
    }
}

impl From<Vec<u8>> for LengthPrefixByteArray {
    fn from(inner: Vec<u8>) -> Self {
        Self(inner)
    }
}

impl McRead for LengthPrefixByteArray {
    fn read<R: Read>(reader: &mut R) -> Result<(Self, usize)> {
        let (buffer_len, len_len) = v32::read(reader)?;

        let mut buffer = vec![0; *buffer_len as usize];
        reader.read_exact(&mut buffer)?;

        Ok((
            LengthPrefixByteArray(buffer),
            *buffer_len as usize + len_len,
        ))
    }
}

impl McWrite for LengthPrefixByteArray {
    fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        v32(self.0.len() as i32).write(writer)?;
        writer.write_all(&self.0)
    }
}

macro_rules! v32_enum_read_write {
    ($enum:ty => $unknown:ident { $($variant:ident = $val:expr),* $(,)? }) => {
        impl crate::types::McRead for $enum {
            fn read<R: std::io::Read>(reader: &mut R) -> std::io::Result<(Self, usize)> {
                let (val, size) = crate::types::v32::read(reader)?;

                let r#enum = match val.0 {
                    $($val => Self::$variant,)*
                    unknown => Self::$unknown(unknown),
                };

                Ok((r#enum, size))
            }
        }

        impl crate::types::McWrite for $enum {
            fn write<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
                let value = match self {
                    $(Self::$variant => $val,)*
                    Self::$unknown(unknown) => *unknown,
                };

                crate::types::v32(value).write(writer)
            }
        }
    };
}

pub(crate) use v32_enum_read_write;
