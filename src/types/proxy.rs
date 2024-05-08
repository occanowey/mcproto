use super::BufType;
use crate::error::Result;
use crate::types::{check_remaining, check_remaining_mut};
use bytes::{Buf, BufMut};

pub mod i32_as_v32 {
    use super::{super::v32, *};

    pub fn buf_read<B: Buf>(buf: &mut B) -> Result<(i32, usize)> {
        let (value, value_length) = v32::buf_read(buf)?;
        Ok((value.0, value_length))
    }

    pub fn buf_write<B: BufMut>(value: &i32, buf: &mut B) -> Result<()> {
        v32(*value).buf_write(buf)
    }
}

pub mod bool_option {
    use super::*;

    pub fn buf_read<B: Buf, T: BufType>(buf: &mut B) -> Result<(Option<T>, usize)> {
        let (has_value, mut total_value_len) = bool::buf_read(buf)?;

        let value = if has_value {
            let (value, value_len) = T::buf_read(buf)?;
            total_value_len += value_len;
            Some(value)
        } else {
            None
        };

        Ok((value, total_value_len))
    }

    pub fn buf_write<B: BufMut, T: BufType>(value: &Option<T>, buf: &mut B) -> Result<()> {
        value.is_some().buf_write(buf)?;

        if let Some(value) = value {
            value.buf_write(buf)?;
        }

        Ok(())
    }
}

pub mod length_prefix_bytes {
    use super::*;

    pub fn buf_read<B: Buf>(buf: &mut B) -> Result<(Vec<u8>, usize)> {
        let (buf_len, buf_len_len) = i32_as_v32::buf_read(buf)?;
        check_remaining(buf, buf_len as _)?;

        let mut bytes = vec![0, buf_len as _];
        buf.copy_to_slice(&mut bytes);

        Ok((bytes, buf_len as usize + buf_len_len))
    }

    pub fn buf_write<B: BufMut, BA: AsRef<[u8]>>(bytes: BA, buf: &mut B) -> Result<()> {
        let bytes = bytes.as_ref();

        i32_as_v32::buf_write(&(bytes.len() as _), buf)?;
        check_remaining_mut(buf, bytes.len())?;
        buf.put_slice(bytes);
        Ok(())
    }
}

pub mod length_prefix_array {
    use super::*;

    pub fn buf_read<B: Buf, T: BufType>(buf: &mut B) -> Result<(Vec<T>, usize)> {
        let mut values = Vec::new();

        let (values_count, mut length) = i32_as_v32::buf_read(buf)?;
        for _ in 0..values_count {
            let (value, value_length) = T::buf_read(buf)?;

            values.push(value);
            length += value_length;
        }

        Ok((values, length))
    }

    pub fn buf_write<B: BufMut, T: BufType, A: AsRef<[T]>>(value: A, buf: &mut B) -> Result<()> {
        let array = value.as_ref();

        let values_count = array.len();
        // TODO: return error rather than panic
        assert!(values_count < (super::super::v32::MAX as usize));
        i32_as_v32::buf_write(&(values_count as _), buf)?;

        for value in array {
            value.buf_write(buf)?;
        }

        Ok(())
    }
}

pub mod remaining_bytes {
    use super::*;

    pub fn buf_read<B: Buf>(buf: &mut B) -> Result<(Vec<u8>, usize)> {
        let len = buf.remaining();
        let mut bytes = vec![0; len];
        buf.copy_to_slice(&mut bytes);
        Ok((bytes, len))
    }

    pub fn buf_write<B: BufMut, BA: AsRef<[u8]>>(bytes: BA, buf: &mut B) -> Result<()> {
        let bytes = bytes.as_ref();
        check_remaining_mut(buf, bytes.len())?;
        buf.put_slice(bytes);
        Ok(())
    }
}
