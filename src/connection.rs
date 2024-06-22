use std::io::{Read, Write};
use std::marker::PhantomData;
use std::net::{Shutdown, TcpStream};

use aes::cipher::{BlockDecryptMut, BlockEncryptMut};
use bytes::{Buf, BufMut, Bytes, BytesMut};
use crypto_common::generic_array::GenericArray;
use crypto_common::KeyIvInit;
use flate2::write::{ZlibDecoder, ZlibEncoder};
use flate2::Compression;
use tracing::{debug, trace};

use crate::error::{Error, Result};
use crate::handshake;
use crate::role::ConnectionRole;
use crate::state::{NextProtocolState, ProtocolState, RoleStateReadPacket, RoleStateWritePacket};
use crate::types::proxy::i32_as_v32;
use crate::types::ReadError;

// would rather this be in connection but generics makes that difficult if not impossible
pub fn connection_from_stream<Role: ConnectionRole>(
    stream: TcpStream,
) -> Result<Connection<Role, handshake::HandshakingState>> {
    Ok(Connection {
        stream,
        recv_buffer: BytesMut::new(),
        ciphers: None,
        compression_threshold: None,

        _role: PhantomData,
        _state: PhantomData,
    })
}

pub struct Connection<Role: ConnectionRole, State: ProtocolState> {
    stream: TcpStream,
    recv_buffer: BytesMut,
    ciphers: Option<(cfb8::Encryptor<aes::Aes128>, cfb8::Decryptor<aes::Aes128>)>,
    compression_threshold: Option<usize>,

    _role: PhantomData<Role>,
    _state: PhantomData<State>,
}

impl<Role: ConnectionRole, State: ProtocolState> Connection<Role, State> {
    fn ensure_next_length(&mut self) -> Result<()> {
        loop {
            match i32_as_v32::buf_read(&mut self.recv_buffer.clone()) {
                Ok(length) => {
                    if self.recv_buffer.remaining() >= length as _ {
                        return Ok(());
                    }
                }
                Err(ReadError::ReadOutOfBounds(..)) => (),
                Err(other) => Err(other)?,
            };

            let mut buf = [0; 512];
            // TODO: timeout
            let len = self.stream.read(&mut buf)?;
            if len == 0 {
                return Err(Error::StreamShutdown);
            }

            self.recv_buffer.extend_from_slice(&buf[0..len]);

            if let Some((_, cipher)) = &mut self.ciphers {
                // TODO: this was copied from the old enc struct, should check if theres a better way to do this

                // safe as long as `<cfb8::Decryptor as BlockSizeUser>::BlockSize == typenum::U1`
                // which is true as of 0.8.1
                let start = self.recv_buffer.len() - len;
                let blocks: &mut [GenericArray<u8, crypto_common::typenum::U1>] =
                    unsafe { std::mem::transmute(&mut self.recv_buffer[start..]) };

                cipher.decrypt_blocks_mut(blocks);
            }
        }
    }

    pub fn read<Packet: RoleStateReadPacket<Role, State> + std::fmt::Debug>(
        &mut self,
    ) -> Result<Packet> {
        let (id, mut data) = self.read_id_body()?;
        if id != Packet::PACKET_ID {
            return Err(Error::IncorectPacketId(Packet::PACKET_ID, id));
        }

        let packet = Packet::read_body(&mut data);
        debug!(state = ?State::LABEL, ?packet, "read packet");
        Ok(packet?)
    }

    pub fn read_id_body(&mut self) -> Result<(i32, Bytes)> {
        self.ensure_next_length()?;
        let length = i32_as_v32::buf_read(&mut self.recv_buffer)? as usize;

        // compression
        let mut body = if self.compression_threshold.is_some() {
            let (body_length, body_length_length) =
                i32_as_v32::buf_read_len(&mut self.recv_buffer).unwrap();

            let compressed_body = self.recv_buffer.split_to(length - body_length_length);
            if body_length > 0 {
                let body = BytesMut::with_capacity(body_length as _);
                let mut decoder = ZlibDecoder::new(body.writer());
                // TODO: check unwrap safety
                decoder.write_all(&compressed_body).unwrap();
                decoder.finish().unwrap().into_inner()
            } else {
                compressed_body
            }
        } else {
            self.recv_buffer.split_to(length)
        };

        let id = i32_as_v32::buf_read(&mut body).unwrap();
        let body = body.freeze();

        trace!(id, ?body, "read id body");
        Ok((id, body))
    }

    pub fn write<Packet: RoleStateWritePacket<Role, State> + std::fmt::Debug>(
        &mut self,
        packet: Packet,
    ) -> Result<()> {
        debug!(state = ?State::LABEL, ?packet, compression = ?self.compression_threshold, "writing packet");

        // id + body
        let mut packet_data = BytesMut::new();
        i32_as_v32::buf_write(&Packet::PACKET_ID, &mut packet_data);
        packet.write_body(&mut packet_data);

        // compression
        let mut compressed_data = if let Some(threshold) = self.compression_threshold {
            let mut data = BytesMut::new();

            if packet_data.len() >= threshold {
                i32_as_v32::buf_write(&(packet_data.len() as _), &mut data);

                let mut encoder = ZlibEncoder::new(data.writer(), Compression::default());
                // TODO: check unwrap safety
                encoder.write_all(&packet_data).unwrap();
                encoder.finish().unwrap().into_inner()
            } else {
                i32_as_v32::buf_write(&0, &mut data);
                data.put(packet_data);
                data
            }
        } else {
            packet_data
        };

        // length + data
        let mut data = BytesMut::new();
        i32_as_v32::buf_write(&(compressed_data.len() as _), &mut data);
        data.put(&mut compressed_data);

        // encryption
        if let Some((cipher, _)) = &mut self.ciphers {
            // TODO: this was copied from the old enc struct, should check if theres a better way to do this

            // safe as long as `<cfb8::Encryptor as BlockSizeUser>::BlockSize == typenum::U1`
            // which is true as of 0.8.1
            let blocks = unsafe {
                &mut *(&mut data as &mut [u8] as *mut [u8]
                    as *mut [GenericArray<u8, crypto_common::typenum::U1>])
            };

            cipher.encrypt_blocks_mut(blocks);
        }

        // write
        self.stream.write_all(&data)?;
        Ok(())
    }

    pub fn set_compression_threshold<T: Into<Option<usize>>>(&mut self, threshold: T) {
        let threshold = threshold.into();
        debug!(state = ?State::LABEL, ?threshold, "setting threshold");
        self.compression_threshold = threshold;
    }

    pub fn set_encryption_secret(&mut self, secret: &[u8]) {
        debug!(state = ?State::LABEL, "setting encryption");

        let encryption_cipher = cfb8::Encryptor::new(secret.into(), secret.into());
        let decryption_cipher = cfb8::Decryptor::new(secret.into(), secret.into());

        self.ciphers.replace((encryption_cipher, decryption_cipher));
    }

    pub fn next_state<NextState: NextProtocolState<State>>(self) -> Connection<Role, NextState> {
        debug!(state = ?State::LABEL, "switching to {:?}", NextState::LABEL);

        Connection {
            stream: self.stream,
            recv_buffer: self.recv_buffer,
            ciphers: self.ciphers,
            compression_threshold: self.compression_threshold,

            _role: PhantomData,
            _state: PhantomData,
        }
    }

    pub fn close(self) -> Result<()> {
        Ok(self.stream.shutdown(Shutdown::Both)?)
    }

    pub fn into_stream(self) -> TcpStream {
        self.stream
    }
}
