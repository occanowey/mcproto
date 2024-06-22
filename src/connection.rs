use bytes::{Buf, BufMut, Bytes, BytesMut};
use std::{io::Write, marker::PhantomData};

use aes::cipher::{BlockDecryptMut, BlockEncryptMut};
use crypto_common::{generic_array::GenericArray, KeyIvInit};
use flate2::{
    write::{ZlibDecoder, ZlibEncoder},
    Compression,
};

use tracing::{debug, trace};

use crate::{
    handshake,
    packet::{self, prelude::i32_as_v32, PacketFromIdBody},
    role::ConnectionRole,
    state::{NextProtocolState, ProtocolState, RoleStatePackets, RoleStateWritePacket},
    types::ReadError,
};

pub enum Event<Role: ConnectionRole, State: ProtocolState>
where
    State: RoleStatePackets<Role>,
{
    NeedMoreData,
    Packet(State::RecvPacket),
}

pub struct Connection<Role: ConnectionRole, State: ProtocolState> {
    recv_buffer: BytesMut,
    ciphers: Option<(cfb8::Encryptor<aes::Aes128>, cfb8::Decryptor<aes::Aes128>)>,
    compression_threshold: Option<usize>,

    _role: PhantomData<Role>,
    _state: PhantomData<State>,
}

pub fn create_new_connection<Role: ConnectionRole>() -> Connection<Role, handshake::HandshakingState>
{
    create_connection()
}

pub fn create_connection<Role: ConnectionRole, State: ProtocolState>() -> Connection<Role, State> {
    Connection {
        recv_buffer: BytesMut::new(),
        ciphers: None,
        compression_threshold: None,

        _role: std::marker::PhantomData,
        _state: std::marker::PhantomData,
    }
}

impl<Role: ConnectionRole, State: ProtocolState> Connection<Role, State> {
    pub fn set_encryption_secret(&mut self, secret: &[u8]) {
        let encryption_cipher = cfb8::Encryptor::new(secret.into(), secret.into());
        let decryption_cipher = cfb8::Decryptor::new(secret.into(), secret.into());

        self.ciphers.replace((encryption_cipher, decryption_cipher));
    }

    pub fn set_compression_threshold<T: Into<Option<usize>>>(&mut self, threshold: T) {
        let threshold = threshold.into();
        self.compression_threshold = threshold;
    }

    pub fn send<Packet: RoleStateWritePacket<Role, State>>(&mut self, packet: Packet) -> Bytes {
        // id + packet data
        let mut packet_data = BytesMut::new();
        i32_as_v32::buf_write(&Packet::PACKET_ID, &mut packet_data);
        packet.write_body(&mut packet_data);

        trace!(id = Packet::PACKET_ID, data = ?packet_data, "send");

        self.send_packet_data(packet_data)
    }

    pub fn send_id_body(&mut self, id: i32, body: &mut Bytes) -> Bytes {
        // id + packet data
        let mut packet_data = BytesMut::new();
        i32_as_v32::buf_write(&id, &mut packet_data);
        packet_data.put(body);

        trace!(id, data = ?packet_data, "send");

        self.send_packet_data(packet_data)
    }

    fn send_packet_data(&mut self, packet_data: BytesMut) -> Bytes {
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

        data.freeze()
    }

    pub fn next_state<NextState: NextProtocolState<State>>(self) -> Connection<Role, NextState> {
        debug!(state = ?State::LABEL, "switching to {:?}", NextState::LABEL);

        Connection {
            recv_buffer: self.recv_buffer,
            ciphers: self.ciphers,
            compression_threshold: self.compression_threshold,

            _role: PhantomData,
            _state: PhantomData,
        }
    }

    pub fn into_bytes(self) -> Bytes {
        self.recv_buffer.freeze()
    }
}

impl<Role: ConnectionRole, State: ProtocolState> Connection<Role, State>
where
    State: RoleStatePackets<Role>,
    State::RecvPacket: packet::PacketFromIdBody,
{
    pub fn recieve_data(&mut self, data: &[u8]) {
        self.recv_buffer.put_slice(data);

        if let Some((_, cipher)) = &mut self.ciphers {
            // TODO: this was copied from the old enc struct, should check if theres a better way to do this

            // safe as long as `<cfb8::Decryptor as BlockSizeUser>::BlockSize == typenum::U1`
            // which is true as of 0.8.1
            let start = self.recv_buffer.len() - data.len();
            let blocks: &mut [GenericArray<u8, crypto_common::typenum::U1>] =
                unsafe { std::mem::transmute(&mut self.recv_buffer[start..]) };

            cipher.decrypt_blocks_mut(blocks);
        }
    }

    pub fn next_event(&mut self) -> Result<Event<Role, State>, ReadError> {
        let mut recv_buffer = self.recv_buffer.clone();

        let length = match i32_as_v32::buf_read(&mut recv_buffer) {
            Ok(length) => length as usize,
            Err(ReadError::ReadOutOfBounds(..)) => return Ok(Event::NeedMoreData),
            Err(other) => return Err(other),
        };

        if recv_buffer.remaining() < length {
            return Ok(Event::NeedMoreData);
        }

        let mut data = if self.compression_threshold.is_some() {
            let (data_length, data_length_length) =
                i32_as_v32::buf_read_len(&mut recv_buffer).unwrap();

            let compressed_data = recv_buffer.split_to(length - data_length_length);
            if data_length > 0 {
                let data = BytesMut::with_capacity(data_length as _);
                let mut decoder = ZlibDecoder::new(data.writer());
                // TODO: check unwrap safety
                decoder.write_all(&compressed_data).unwrap();
                decoder.finish().unwrap().into_inner()
            } else {
                compressed_data
            }
        } else {
            recv_buffer.split_to(length)
        };

        let id = i32_as_v32::buf_read(&mut data).unwrap();

        let body = data.freeze();
        trace!(id, ?body, "next event");
        let packet = State::RecvPacket::from_id_body(id, body)?;

        std::mem::swap(&mut recv_buffer, &mut self.recv_buffer);
        Ok(Event::Packet(packet))
    }
}
