use std::io::{Read, Write};

use aes::cipher::{BlockDecryptMut, BlockEncryptMut};
use crypto_common::{generic_array::GenericArray, KeyIvInit};
use tracing::trace;

use aes::Aes128;

pub struct EncryptableBufReader<R: Read> {
    inner: R,
    cipher: Option<cfb8::Decryptor<Aes128>>,

    buffer: Vec<u8>,
    buffer_offset: usize,
    buffer_len: usize,
}

impl<R: Read> EncryptableBufReader<R> {
    pub fn wrap(inner: R) -> Self {
        EncryptableBufReader {
            inner,
            cipher: None,
            buffer: vec![0; 128],
            buffer_offset: 0,
            buffer_len: 0,
        }
    }

    pub fn set_secret(&mut self, secret: &[u8]) {
        let cipher = cfb8::Decryptor::new(secret.into(), secret.into());
        self.cipher.replace(cipher);
    }
}

impl<R: Read> Read for EncryptableBufReader<R> {
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
            // safe as long as `<cfb8::Encryptor as BlockSizeUser>::BlockSize == typenum::U1`
            // which is true as of 0.8.1
            let blocks: &mut [GenericArray<u8, crypto_common::typenum::U1>] =
                unsafe { std::mem::transmute(&mut buf[..out_len]) };

            cipher.decrypt_blocks_mut(blocks);
        }

        trace!(
            was_encrypted = self.cipher.is_some(),
            "read data {:?}",
            &buf[..out_len]
        );

        Ok(out_len)
    }
}

pub struct EncryptableWriter<W: Write> {
    inner: W,
    cipher: Option<cfb8::Encryptor<Aes128>>,
}

impl<W: Write> EncryptableWriter<W> {
    pub fn wrap(inner: W) -> Self {
        EncryptableWriter {
            inner,
            cipher: None,
        }
    }

    pub fn set_secret(&mut self, secret: &[u8]) {
        let cipher = cfb8::Encryptor::new(secret.into(), secret.into());
        self.cipher.replace(cipher);
    }
}

impl<W: Write> Write for EncryptableWriter<W> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        trace!(
            to_be_encrypted = self.cipher.is_some(),
            "write data {:?}",
            &buf
        );

        let mut buf = buf.to_vec();
        if let Some(cipher) = &mut self.cipher {
            // safe as long as `<cfb8::Encryptor as BlockSizeUser>::BlockSize == typenum::U1`
            // which is true as of 0.8.1
            let blocks = unsafe {
                &mut *(buf.as_mut_slice() as *mut [u8]
                    as *mut [GenericArray<u8, crypto_common::typenum::U1>])
            };

            cipher.encrypt_blocks_mut(blocks);
        }

        self.inner.write_all(&buf)?;
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.inner.flush()
    }
}
