use std::io::Read;
use std::marker::PhantomData;
use std::net::{Shutdown, TcpStream};

use aes::cipher::NewCipher;
use aes::Aes128;
use cfb8::Cfb8;
use flate2::read::ZlibDecoder;
use tracing::{debug, trace};

use super::encryption::{EncryptableBufReader, EncryptableWriter};
use crate::error::{Error, Result};
use crate::net::{
    side::NetworkSide,
    state::{
        ConfigurationState, HandshakingState, LoginState, NetworkState, PlayState,
        SidedStateReadPacket, SidedStateWritePacket, StatusState,
    },
};
use crate::{PacketBuilder, ReadExt};

pub type Cipher = Cfb8<Aes128>;

// would rather this be in network handler but generics makes that difficult if not impossible
pub fn handler_from_stream<D: NetworkSide>(
    stream: TcpStream,
) -> Result<NetworkHandler<D, HandshakingState>> {
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

pub struct NetworkHandler<D: NetworkSide, S: NetworkState> {
    stream: TcpStream,
    _side: PhantomData<D>,
    _state: PhantomData<S>,

    compression: Option<usize>,

    reader: EncryptableBufReader<TcpStream, Cipher>,
    writer: EncryptableWriter<TcpStream, Cipher>,
}

impl<D: NetworkSide, S: NetworkState> NetworkHandler<D, S> {
    pub fn read<P: SidedStateReadPacket<D, S> + std::fmt::Debug>(&mut self) -> Result<P> {
        let (id, data) = self.read_raw_data()?;
        if id != P::PACKET_ID {
            return Err(Error::IncorectPacketId(P::PACKET_ID, id));
        }

        let packet = P::read_data(&mut data.as_slice(), data.len());
        debug!(state = ?S::LABEL, ?packet, "read packet");
        packet
    }

    pub fn read_raw_data(&mut self) -> Result<(i32, Vec<u8>)> {
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
        Ok((id, buffer))
    }

    pub fn write<P: SidedStateWritePacket<D, S> + std::fmt::Debug>(
        &mut self,
        packet: P,
    ) -> Result<()> {
        debug!(state = ?S::LABEL, ?packet, compression = ?self.compression, "writing packet");

        let mut builder = PacketBuilder::new(P::PACKET_ID)?;
        packet.write_data(&mut builder)?;

        if let Some(threshold) = self.compression {
            Ok(builder.write_compressed(&mut self.writer, threshold)?)
        } else {
            Ok(builder.write_to(&mut self.writer)?)
        }
    }

    pub fn set_compression_threshold<T: Into<Option<usize>>>(&mut self, threshold: T) {
        let threshold = threshold.into();
        debug!(state = ?S::LABEL, ?threshold, "setting threshold");
        self.compression = threshold;
    }

    pub fn set_encryption_secret(&mut self, secret: &[u8]) {
        debug!(state = ?S::LABEL, "setting encryption");

        let read_cipher = Cipher::new_from_slices(secret, secret).unwrap();
        self.reader.set_cipher(read_cipher);

        let write_cipher = Cipher::new_from_slices(secret, secret).unwrap();
        self.writer.set_cipher(write_cipher);
    }

    pub fn close(self) -> Result<()> {
        Ok(self.stream.shutdown(Shutdown::Both)?)
    }

    pub fn into_stream(self) -> TcpStream {
        self.stream
    }
}

macro_rules! same_fields_different_generics {
    ($self:ident) => {
        NetworkHandler {
            stream: $self.stream,
            _side: PhantomData,
            _state: PhantomData,

            compression: $self.compression,

            reader: $self.reader,
            writer: $self.writer,
        }
    };
}

impl<D: NetworkSide> NetworkHandler<D, HandshakingState> {
    pub fn status(self) -> NetworkHandler<D, StatusState> {
        debug!(state = ?HandshakingState::LABEL, "switching to status state");
        same_fields_different_generics!(self)
    }

    pub fn login(self) -> NetworkHandler<D, LoginState> {
        debug!(state = ?HandshakingState::LABEL, "switching to login state");
        same_fields_different_generics!(self)
    }
}

impl<D: NetworkSide> NetworkHandler<D, LoginState> {
    pub fn configuration(self) -> NetworkHandler<D, ConfigurationState> {
        debug!(state = ?LoginState::LABEL, "switching to configuration state");
        same_fields_different_generics!(self)
    }
}

impl<D: NetworkSide> NetworkHandler<D, ConfigurationState> {
    pub fn play(self) -> NetworkHandler<D, PlayState> {
        debug!(state = ?ConfigurationState::LABEL, "switching to play state");
        same_fields_different_generics!(self)
    }
}
