use super::ensure_remaining;
use super::BufType;
use super::Result;
use bytes::{Buf, BufMut};

pub mod i32_as_v32 {
    use super::{super::v32, *};

    pub fn buf_read<B: Buf>(buf: &mut B) -> Result<i32> {
        self::buf_read_len(buf).map(|value| value.0)
    }

    pub fn buf_read_len<B: Buf>(buf: &mut B) -> Result<(i32, usize)> {
        v32::buf_read_len(buf).map(|(value, len)| (value.0, len))
    }

    pub fn buf_write<B: BufMut>(value: &i32, buf: &mut B) {
        v32(*value).buf_write(buf)
    }
}

pub mod bool_option {
    use super::*;

    pub fn buf_read<B: Buf, T: BufType>(buf: &mut B) -> Result<Option<T>> {
        self::buf_read_len(buf).map(|value| value.0)
    }

    pub fn buf_read_len<B: Buf, T: BufType>(buf: &mut B) -> Result<(Option<T>, usize)> {
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

    pub fn buf_write<B: BufMut, T: BufType>(value: &Option<T>, buf: &mut B) {
        value.is_some().buf_write(buf);

        if let Some(value) = value {
            value.buf_write(buf);
        }
    }
}

pub mod length_prefix_bytes {
    use super::*;

    pub fn buf_read<B: Buf>(buf: &mut B) -> Result<Vec<u8>> {
        self::buf_read_len(buf).map(|value| value.0)
    }

    pub fn buf_read_len<B: Buf>(buf: &mut B) -> Result<(Vec<u8>, usize)> {
        let (buf_len, buf_len_len) = i32_as_v32::buf_read_len(buf)?;
        ensure_remaining(buf, buf_len as _)?;

        let mut bytes = vec![0; buf_len as _];
        buf.copy_to_slice(&mut bytes);

        Ok((bytes, buf_len as usize + buf_len_len))
    }

    pub fn buf_write<B: BufMut, BA: AsRef<[u8]>>(bytes: BA, buf: &mut B) {
        let bytes = bytes.as_ref();

        i32_as_v32::buf_write(&(bytes.len() as _), buf);
        buf.put_slice(bytes);
    }
}

pub mod length_prefix_array {
    use super::*;

    pub fn buf_read<B: Buf, T: BufType>(buf: &mut B) -> Result<Vec<T>> {
        self::buf_read_len(buf).map(|value| value.0)
    }

    pub fn buf_read_len<B: Buf, T: BufType>(buf: &mut B) -> Result<(Vec<T>, usize)> {
        let mut values = Vec::new();

        let (values_count, mut length) = i32_as_v32::buf_read_len(buf)?;
        for _ in 0..values_count {
            let (value, value_length) = T::buf_read_len(buf)?;

            values.push(value);
            length += value_length;
        }

        Ok((values, length))
    }

    pub fn buf_write<B: BufMut, T: BufType, A: AsRef<[T]>>(value: A, buf: &mut B) {
        let array = value.as_ref();

        let values_count = array.len();
        // TODO: return error rather than panic
        assert!(values_count < (super::super::v32::MAX as usize));
        i32_as_v32::buf_write(&(values_count as _), buf);

        for value in array {
            value.buf_write(buf);
        }
    }
}

pub mod remaining_bytes {
    use super::*;

    pub fn buf_read<B: Buf>(buf: &mut B) -> Result<Vec<u8>> {
        self::buf_read_len(buf).map(|value| value.0)
    }

    pub fn buf_read_len<B: Buf>(buf: &mut B) -> Result<(Vec<u8>, usize)> {
        let len = buf.remaining();
        let mut bytes = vec![0; len];
        buf.copy_to_slice(&mut bytes);
        Ok((bytes, len))
    }

    pub fn buf_write<B: BufMut, BA: AsRef<[u8]>>(bytes: BA, buf: &mut B) {
        buf.put_slice(bytes.as_ref());
    }
}
