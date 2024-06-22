use std::io::{Read, Write};
use std::marker::PhantomData;
use std::net::{Shutdown, TcpStream};

use bytes::{BufMut, Bytes, BytesMut};
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use tracing::{debug, trace};

use super::encryption::{EncryptableBufReader, EncryptableWriter};
use crate::error::{Error, Result};
use crate::handshake;
use crate::net::side::NetworkSide;
use crate::state::{NetworkState, NextHandlerState, SidedStateReadPacket, SidedStateWritePacket};
use crate::types::proxy::i32_as_v32;
use crate::varint::VarintReadExt;

// would rather this be in network handler but generics makes that difficult if not impossible
pub fn handler_from_stream<Side: NetworkSide>(
    stream: TcpStream,
) -> Result<NetworkHandler<Side, handshake::HandshakingState>> {
    let reader = EncryptableBufReader::wrap(stream.try_clone()?);
    let writer = EncryptableWriter::wrap(stream.try_clone()?);

    Ok(NetworkHandler {
        stream,
        _side: PhantomData,
        _state: PhantomData,

        compression: None,

        reader,
        writer,
    })
}

pub struct NetworkHandler<Side: NetworkSide, State: NetworkState> {
    stream: TcpStream,
    _side: PhantomData<Side>,
    _state: PhantomData<State>,

    compression: Option<usize>,

    reader: EncryptableBufReader<TcpStream>,
    writer: EncryptableWriter<TcpStream>,
}

impl<Side: NetworkSide, State: NetworkState> NetworkHandler<Side, State> {
    pub fn read<Packet: SidedStateReadPacket<Side, State> + std::fmt::Debug>(
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
        let (length, _) = self.reader.read_varint()?;

        // allocate and read first length (packet length or compressed length)
        let mut buffer = vec![0; length as usize];
        self.reader.read_exact(&mut buffer)?;

        let mut buffer = if self.compression.is_some() {
            let (data_length, dl_len) = buffer.as_slice().read_varint()?;
            buffer.drain(0..dl_len);

            if data_length > 0 {
                let mut decoder = ZlibDecoder::new(buffer.as_slice());

                let mut buffer = vec![0; data_length as usize];
                decoder.read_exact(&mut buffer)?;

                buffer
            } else {
                buffer
            }
        } else {
            buffer
        };

        let (id, id_len) = buffer.as_slice().read_varint()?;
        buffer.drain(0..id_len);

        trace!(id, ?buffer, "read raw packet");
        Ok((id, Bytes::from(buffer)))
    }

    pub fn write<Packet: SidedStateWritePacket<Side, State> + std::fmt::Debug>(
        &mut self,
        packet: Packet,
    ) -> Result<()> {
        debug!(state = ?State::LABEL, ?packet, compression = ?self.compression, "writing packet");

        // id + body
        let mut packet_data = BytesMut::new();
        i32_as_v32::buf_write(&Packet::PACKET_ID, &mut packet_data);
        packet.write_body(&mut packet_data);

        // compression
        let mut compressed_data = if let Some(threshold) = self.compression {
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

        // write
        self.writer.write_all(&data)?;
        Ok(())
    }

    pub fn set_compression_threshold<T: Into<Option<usize>>>(&mut self, threshold: T) {
        let threshold = threshold.into();
        debug!(state = ?State::LABEL, ?threshold, "setting threshold");
        self.compression = threshold;
    }

    pub fn set_encryption_secret(&mut self, secret: &[u8]) {
        debug!(state = ?State::LABEL, "setting encryption");

        self.reader.set_secret(secret);
        self.writer.set_secret(secret);
    }

    pub fn next_state<NextState: NextHandlerState<State>>(self) -> NetworkHandler<Side, NextState> {
        debug!(state = ?State::LABEL, "switching to {:?}", NextState::LABEL);

        NetworkHandler {
            stream: self.stream,
            _side: PhantomData,
            _state: PhantomData,

            compression: self.compression,

            reader: self.reader,
            writer: self.writer,
        }
    }

    pub fn close(self) -> Result<()> {
        Ok(self.stream.shutdown(Shutdown::Both)?)
    }

    pub fn into_stream(self) -> TcpStream {
        self.stream
    }
}
