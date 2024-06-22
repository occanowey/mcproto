use std::ops::Deref;

use bytes::{Buf, BufMut};
use uuid::Uuid;

pub mod proxy;

use self::proxy::length_prefix_bytes;

#[derive(Debug, thiserror::Error)]
pub enum ReadError {
    #[error("read out of bounds: len is: {0}, but tried to read: {1}")]
    ReadOutOfBounds(usize, usize),

    #[error("utf8 error: {0}")]
    Utf8Error(#[from] std::string::FromUtf8Error),

    #[error("varint too large")]
    VarIntTooLarge,
}

type Result<T> = std::result::Result<T, ReadError>;

pub trait BufType: Sized {
    fn buf_read<B: Buf>(buf: &mut B) -> Result<Self> {
        Self::buf_read_len(buf).map(|value| value.0)
    }

    fn buf_read_len<B: Buf>(buf: &mut B) -> Result<(Self, usize)>;

    fn buf_write<B: BufMut>(&self, buf: &mut B);
}

macro_rules! impl_primitive {
    ($self:ty, $get_fn:ident, $put_fn:ident) => {
        impl BufType for $self {
            fn buf_read_len<B: Buf>(buf: &mut B) -> Result<(Self, usize)> {
                const SIZE: usize = core::mem::size_of::<$self>();
                ensure_remaining(buf, SIZE)?;

                Ok((buf.$get_fn(), SIZE))
            }

            fn buf_write<B: BufMut>(&self, buf: &mut B) {
                buf.$put_fn(*self)
            }
        }
    };
}

pub(crate) fn ensure_remaining<B: Buf>(buf: &B, size: usize) -> Result<()> {
    if buf.remaining() < size {
        return Err(ReadError::ReadOutOfBounds(buf.remaining(), size));
    }

    Ok(())
}

// Boolean
impl BufType for bool {
    fn buf_read_len<B: Buf>(buf: &mut B) -> Result<(Self, usize)> {
        ensure_remaining(buf, 1)?;
        Ok((buf.get_u8() != 0, 1))
    }

    fn buf_write<B: BufMut>(&self, buf: &mut B) {
        buf.put_u8(*self as _);
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
    fn buf_read_len<B: Buf>(buf: &mut B) -> Result<(Self, usize)> {
        let (bytes, len) = length_prefix_bytes::buf_read_len(buf)?;
        let string = String::from_utf8(bytes)?;

        Ok((string, len))
    }

    fn buf_write<B: BufMut>(&self, buf: &mut B) {
        proxy::i32_as_v32::buf_write(&(self.len() as _), buf);

        buf.put(self.as_bytes());
    }
}

// Text Component
#[derive(Debug)]
pub enum TextComponent {
    String(String),
    Compound(()),
}

impl BufType for TextComponent {
    fn buf_read_len<B: Buf>(_buf: &mut B) -> Result<(Self, usize)> {
        todo!()
    }

    fn buf_write<B: BufMut>(&self, _buf: &mut B) {
        todo!()
    }
}

// Chat

// Identifier
// TODO: make sure it's a valid ident?
#[derive(Debug, Hash, PartialEq, Eq)]
pub struct Identifier(pub String);

impl BufType for Identifier {
    fn buf_read_len<B: Buf>(buf: &mut B) -> Result<(Self, usize)> {
        let (data, length) = String::buf_read_len(buf)?;

        Ok((Identifier(data), length))
    }

    fn buf_write<B: BufMut>(&self, buf: &mut B) {
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
    fn buf_read_len<B: Buf>(buf: &mut B) -> Result<(Self, usize)> {
        let mut acc = 0;
        let mut i = 0;

        loop {
            let byte = u8::buf_read(buf)? as i32;
            acc |= (byte & 0x7F) << (i * 7);

            i += 1;
            if i > 5 {
                return Err(ReadError::VarIntTooLarge);
            }

            if (byte & 0b10000000) == 0 {
                break;
            }
        }

        Ok((v32(acc), i))
    }

    fn buf_write<B: BufMut>(&self, buf: &mut B) {
        let mut input = self.0 as u32;

        loop {
            if (input & 0xFFFFFF80) == 0 {
                break;
            }

            ((input & 0x7F | 0x80) as u8).buf_write(buf);
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
    fn buf_read_len<B: Buf>(buf: &mut B) -> Result<(Self, usize)> {
        ensure_remaining(buf, 16)?;
        let mut buffer = [0; 16];
        buf.copy_to_slice(&mut buffer);

        Ok((Uuid::from_bytes(buffer), buffer.len()))
    }

    fn buf_write<B: BufMut>(&self, buf: &mut B) {
        buf.put_slice(self.as_bytes());
    }
}

// Optional X
impl<T: BufType> BufType for Option<T> {
    fn buf_read_len<B: Buf>(buf: &mut B) -> Result<(Option<T>, usize)> {
        let (has_value, mut total_value_len) = bool::buf_read_len(buf)?;

        let value = if has_value {
            let (value, value_len) = T::buf_read_len(buf)?;
            total_value_len += value_len;
            Some(value)
        } else {
            None
        };

        Ok((value, total_value_len))
    }

    fn buf_write<B: BufMut>(&self, buf: &mut B) {
        self.is_some().buf_write(buf);

        if let Some(value) = self {
            value.buf_write(buf);
        }
    }
}

// Array of X

// X Enum

macro_rules! v32_prefix_enum {
    ($enum:ty => $unknown:ident { $($variant:ident = $val:expr),* $(,)? }) => {
        impl crate::types::BufType for $enum {
            fn buf_read_len<B: bytes::Buf>(buf: &mut B) -> std::result::Result<(Self, usize), crate::types::ReadError> {
                let (val, size) = crate::types::proxy::i32_as_v32::buf_read_len(buf)?;

                let r#enum = match val {
                    $($val => Self::$variant,)*
                    unknown => Self::$unknown(unknown),
                };

                Ok((r#enum, size))
            }

            fn buf_write<B: bytes::BufMut>(&self, buf: &mut B) {
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
