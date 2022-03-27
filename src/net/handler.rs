use std::io::{Error as IoError, Read};
use std::marker::PhantomData;
use std::net::{Shutdown, TcpStream};

use flate2::read::ZlibDecoder;

use crate::net::state::{
    Handshaking, LoginState, NetworkSide, NetworkState, SidedStateReadPacket,
    SidedStateWritePacket, StatusState,
};
use crate::{PacketBuilder, ReadExt};

// would rather this be in network handler but generics makes that difficult if not impossible
pub fn handler_from_stream<D: NetworkSide>(stream: TcpStream) -> NetworkHandler<D, Handshaking> {
    NetworkHandler {
        stream,
        _side: PhantomData,
        _state: PhantomData,

        compression: None,
    }
}

pub struct NetworkHandler<D: NetworkSide, S: NetworkState> {
    stream: TcpStream,
    _side: PhantomData<D>,
    _state: PhantomData<S>,

    compression: Option<usize>,
}

impl<D: NetworkSide, S: NetworkState> NetworkHandler<D, S> {
    pub fn read<P: SidedStateReadPacket<D, S>>(&mut self) -> Result<P, IoError> {
        let (_, data) = self.read_raw_data()?;
        self.read_from_data(&mut data.as_slice())
    }

    pub fn read_raw_data(&mut self) -> Result<(i32, Vec<u8>), IoError> {
        let (length, _) = self.stream.read_varint()?;

        // allocate and read first length (packet length or compressed length)
        let mut buffer = vec![0; length as usize];
        self.stream.read_exact(&mut buffer)?;

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

        Ok((id, buffer))
    }

    pub fn read_from_data<P: SidedStateReadPacket<D, S>>(
        &self,
        data: &mut &[u8],
    ) -> Result<P, IoError> {
        P::read_data(data, data.len())
    }

    pub fn write<P: SidedStateWritePacket<D, S>>(&mut self, packet: P) -> Result<(), IoError> {
        let mut builder = PacketBuilder::new(P::PACKET_ID)?;
        packet.write_data(&mut builder)?;

        if let Some(threshold) = self.compression {
            builder.write_compressed(&mut self.stream, threshold)
        } else {
            builder.write_to(&mut self.stream)
        }
    }

    pub fn set_compression_threshold<T: Into<Option<usize>>>(&mut self, threshold: T) {
        self.compression = threshold.into();
    }

    pub fn close(self) -> Result<(), IoError> {
        self.stream.shutdown(Shutdown::Both)
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
        }
    };
}

impl<D: NetworkSide> NetworkHandler<D, Handshaking> {
    pub fn status(self) -> NetworkHandler<D, StatusState> {
        same_fields_different_generics!(self)
    }

    pub fn login(self) -> NetworkHandler<D, LoginState> {
        same_fields_different_generics!(self)
    }
}
