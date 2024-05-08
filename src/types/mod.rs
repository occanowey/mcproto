use std::ops::Deref;

use bytes::{Buf, BufMut};
use uuid::Uuid;

use crate::error::{Error, Result};

use self::proxy::length_prefix_bytes;

pub mod proxy;

pub trait BufType: Sized {
    // todo: look into if size is needed
    fn buf_read<B: Buf>(buf: &mut B) -> Result<(Self, usize)>;

    fn buf_write<B: BufMut>(&self, buf: &mut B) -> Result<()>;
}

macro_rules! impl_primitive {
    ($self:ty, $get_fn:ident, $put_fn:ident) => {
        impl BufType for $self {
            fn buf_read<B: Buf>(buf: &mut B) -> Result<(Self, usize)> {
                const SIZE: usize = core::mem::size_of::<$self>();
                check_remaining(buf, SIZE)?;

                Ok((buf.$get_fn(), SIZE))
            }

            fn buf_write<B: BufMut>(&self, buf: &mut B) -> Result<()> {
                const SIZE: usize = core::mem::size_of::<$self>();
                check_remaining_mut(buf, SIZE)?;

                Ok(buf.$put_fn(*self))
            }
        }
    };
}

fn check_remaining<B: Buf>(buf: &B, size: usize) -> Result<()> {
    if buf.remaining() < size {
        return Err(Error::ReadOutOfBounds(buf.remaining(), size));
    }

    Ok(())
}

fn check_remaining_mut<B: BufMut>(buf: &B, size: usize) -> Result<()> {
    if buf.remaining_mut() < size {
        return Err(Error::ReadOutOfBounds(buf.remaining_mut(), size));
    }

    Ok(())
}

// Boolean
impl BufType for bool {
    fn buf_read<B: Buf>(buf: &mut B) -> Result<(Self, usize)> {
        check_remaining(buf, 1)?;
        Ok((buf.get_u8() != 0, 1))
    }

    fn buf_write<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        check_remaining_mut(buf, 1)?;
        buf.put_u8(*self as _);
        Ok(())
    }
}

impl_primitive!(i8, get_i8, put_i8);

// Unsigned Byte
impl_primitive!(u8, get_u8, put_u8);

// Short
impl_primitive!(i16, get_i16, put_i16);

// Unsigned Short
impl_primitive!(u16, get_u16, put_u16);

// Int
impl_primitive!(i32, get_i32, put_i32);

// Long
impl_primitive!(i64, get_i64, put_i64);

// Float
impl_primitive!(f32, get_f32, put_f32);

// Double
impl_primitive!(f64, get_f64, put_f64);

// String
impl BufType for String {
    fn buf_read<B: Buf>(buf: &mut B) -> Result<(Self, usize)> {
        let (bytes, len) = length_prefix_bytes::buf_read(buf)?;
        let string = String::from_utf8(bytes)?;

        Ok((string, len))
    }

    fn buf_write<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        proxy::i32_as_v32::buf_write(&(self.len() as _), buf)?;
        check_remaining_mut(buf, self.len())?;
        buf.put(self.as_bytes());
        Ok(())
    }
}

// Text Component
#[derive(Debug)]
pub enum TextComponent {
    String(String),
    Compound(()),
}

impl BufType for TextComponent {
    fn buf_read<B: Buf>(_buf: &mut B) -> Result<(Self, usize)> {
        todo!()
    }

    fn buf_write<B: BufMut>(&self, _buf: &mut B) -> Result<()> {
        todo!()
    }
}

// Chat

// Identifier
// TODO: make sure it's a valid ident?
#[derive(Debug, Hash, PartialEq, Eq)]
pub struct Identifier(pub String);

impl BufType for Identifier {
    fn buf_read<B: Buf>(buf: &mut B) -> Result<(Self, usize)> {
        let (data, length) = String::buf_read(buf)?;

        Ok((Identifier(data), length))
    }

    fn buf_write<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        self.0.buf_write(buf)
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

impl BufType for v32 {
    fn buf_read<B: Buf>(buf: &mut B) -> Result<(Self, usize)> {
        let mut acc = 0;
        let mut i = 0;

        loop {
            let byte = u8::buf_read(buf)?.0 as i32;
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

    fn buf_write<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        let mut input = self.0 as u32;

        loop {
            if (input & 0xFFFFFF80) == 0 {
                break;
            }

            ((input & 0x7F | 0x80) as u8).buf_write(buf)?;
            input >>= 7;
        }

        (input as u8).buf_write(buf)
    }
}

// VarLong

// Entity Metadata

// Slot

// NBT Tag

// Position

// Angle

// UUID
impl BufType for Uuid {
    fn buf_read<B: Buf>(buf: &mut B) -> Result<(Self, usize)> {
        check_remaining(buf, 16)?;
        let mut buffer = [0; 16];
        buf.copy_to_slice(&mut buffer);

        Ok((Uuid::from_bytes(buffer), buffer.len()))
    }

    fn buf_write<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        check_remaining_mut(buf, 16)?;
        buf.put_slice(self.as_bytes());
        Ok(())
    }
}

// Optional X

// Array of X

// X Enum

macro_rules! v32_prefix_enum {
    ($enum:ty => $unknown:ident { $($variant:ident = $val:expr),* $(,)? }) => {
        impl crate::types::BufType for $enum {
            fn buf_read<B: bytes::Buf>(buf: &mut B) -> crate::error::Result<(Self, usize)> {
                let (val, size) = crate::types::proxy::i32_as_v32::buf_read(buf)?;

                let r#enum = match val {
                    $($val => Self::$variant,)*
                    unknown => Self::$unknown(unknown),
                };

                Ok((r#enum, size))
            }

            fn buf_write<B: bytes::BufMut>(&self, buf: &mut B) -> crate::error::Result<()> {
                let value = match self {
                    $(Self::$variant => $val,)*
                    Self::$unknown(unknown) => *unknown,
                };

                crate::types::proxy::i32_as_v32::buf_write(&value, buf)
            }
        }
    };
}

pub(crate) use v32_prefix_enum;
