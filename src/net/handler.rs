use std::io::Error as IoError;
use std::marker::PhantomData;
use std::net::{Shutdown, TcpStream};

use crate::net::state::{
    Handshaking, LoginState, NetworkSide, NetworkState, SidedStateReadPacket,
    SidedStateWritePacket, StatusState,
};
use crate::PacketBuilder;

// would rather this be in network handler but generics makes that difficult if not impossible
pub fn handler_from_stream<D: NetworkSide>(stream: TcpStream) -> NetworkHandler<D, Handshaking> {
    NetworkHandler {
        stream,
        _side: PhantomData,
        _state: PhantomData,
    }
}

pub struct NetworkHandler<D: NetworkSide, S: NetworkState> {
    stream: TcpStream,
    _side: PhantomData<D>,
    _state: PhantomData<S>,
}

impl<D: NetworkSide, S: NetworkState> NetworkHandler<D, S> {
    pub fn read<P: SidedStateReadPacket<D, S>>(&mut self) -> Result<P, IoError> {
        P::read(&mut self.stream)
    }

    pub fn write<P: SidedStateWritePacket<D, S>>(&mut self, packet: P) -> Result<(), IoError> {
        let mut builder = PacketBuilder::new(P::PACKET_ID)?;
        packet.write_data(&mut builder)?;
        builder.write_to(&mut self.stream)
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
