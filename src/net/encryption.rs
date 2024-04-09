use std::io::{Read, Write};

use aes::cipher::AsyncStreamCipher;
use tracing::trace;

pub struct EncryptableBufReader<R: Read, C: AsyncStreamCipher> {
    inner: R,
    cipher: Option<C>,

    buffer: Vec<u8>,
    buffer_offset: usize,
    buffer_len: usize,
}

impl<R: Read, C: AsyncStreamCipher> EncryptableBufReader<R, C> {
    pub fn wrap(inner: R) -> Self {
        EncryptableBufReader {
            inner,
            cipher: None,
            buffer: vec![0; 128],
            buffer_offset: 0,
            buffer_len: 0,
        }
    }

    pub fn set_cipher(&mut self, cipher: C) {
        self.cipher.replace(cipher);
    }
}

impl<R: Read, C: AsyncStreamCipher> Read for EncryptableBufReader<R, C> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        if self.buffer_len == 0 {
            self.buffer_len = self.inner.read(&mut self.buffer)?;
            self.buffer_offset = 0;
        }

        let out_len = (self.buffer_len).min(buf.len());
        buf[..out_len]
            .copy_from_slice(&self.buffer[self.buffer_offset..(self.buffer_offset + out_len)]);

        self.buffer_offset += out_len;
        self.buffer_len -= out_len;

        if let Some(cipher) = &mut self.cipher {
            cipher.decrypt(&mut buf[..out_len]);
        }

        trace!(
            was_encrypted = self.cipher.is_some(),
            "read data {:?}",
            &buf[..out_len]
        );

        Ok(out_len)
    }
}

pub struct EncryptableWriter<W: Write, C: AsyncStreamCipher> {
    inner: W,
    cipher: Option<C>,
}

impl<W: Write, C: AsyncStreamCipher> EncryptableWriter<W, C> {
    pub fn wrap(inner: W) -> Self {
        EncryptableWriter {
            inner,
            cipher: None,
        }
    }

    pub fn set_cipher(&mut self, cipher: C) {
        self.cipher.replace(cipher);
    }
}

impl<W: Write, C: AsyncStreamCipher> Write for EncryptableWriter<W, C> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        trace!(
            to_be_encrypted = self.cipher.is_some(),
            "write data {:?}",
            &buf
        );

        let mut buf = buf.to_vec();
        if let Some(cipher) = &mut self.cipher {
            cipher.encrypt(&mut buf);
        }

        self.inner.write_all(&buf)?;
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.inner.flush()
    }
}
