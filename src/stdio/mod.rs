use std::{
    convert::TryInto,
    io::{Read, Write},
    net::{TcpStream, ToSocketAddrs},
};

use bytes::Bytes;

use crate::{connection, error, packet, role, state};

pub struct StdIoConnection<Role, State>
where
    Role: role::ConnectionRole,
    State: state::ProtocolState,
{
    stream: TcpStream,
    connection: connection::Connection<Role, State>,
}

pub fn accept_stdio_stream<Role: role::ConnectionRole, State: state::ProtocolState>(
    stream: TcpStream,
) -> Result<StdIoConnection<Role, State>, error::Error> {
    stream.set_nodelay(true)?;

    Ok(StdIoConnection {
        stream,
        connection: connection::create_connection(),
    })
}

pub fn connect_stdio_stream<
    Addr: ToSocketAddrs,
    Role: role::ConnectionRole,
    State: state::ProtocolState,
>(
    addr: Addr,
) -> Result<StdIoConnection<Role, State>, error::Error> {
    accept_stdio_stream(TcpStream::connect(addr)?)
}

impl<Role, State> StdIoConnection<Role, State>
where
    Role: role::ConnectionRole,
    State: state::ProtocolState,
{
    // TODO: timeout
    pub fn next_packet(&mut self) -> Result<State::RecvPacket, error::Error>
    where
        State: state::RoleStatePackets<Role>,
        State::RecvPacket: packet::PacketFromIdBody,
    {
        loop {
            let event = self.connection.next_event()?;
            match event {
                connection::Event::NeedMoreData => {
                    let mut data = [0; 512];
                    let len = self.stream.read(&mut data)?;
                    if len == 0 {
                        return Err(error::Error::StreamShutdown);
                    }
                    self.connection.recieve_data(&data[0..len]);
                }
                connection::Event::Packet(packet) => return Ok(packet),
            }
        }
    }

    pub fn expect_next_packet<Packet>(&mut self) -> Result<Packet, error::Error>
    where
        Packet: state::RoleStateReadPacket<Role, State>,

        State: state::RoleStatePackets<Role>,
        State::RecvPacket: packet::PacketFromIdBody,
        State::RecvPacket: TryInto<Packet, Error = error::Error>,
    {
        self.next_packet()?.try_into()
    }

    pub fn write_packet<Packet: state::RoleStateWritePacket<Role, State>>(
        &mut self,
        packet: Packet,
    ) -> Result<(), error::Error> {
        Ok(self.stream.write_all(&self.connection.send(packet))?)
    }

    pub fn write_id_body(&mut self, id: i32, body: &mut Bytes) -> Result<(), error::Error> {
        Ok(self
            .stream
            .write_all(&self.connection.send_id_body(id, body))?)
    }

    pub fn next_state<NextState>(self) -> StdIoConnection<Role, NextState>
    where
        NextState: state::ProtocolState,
        NextState: state::NextProtocolState<State>,
    {
        StdIoConnection {
            stream: self.stream,
            connection: self.connection.next_state(),
        }
    }

    pub fn shutdown(&self, how: std::net::Shutdown) -> Result<(), error::Error> {
        Ok(self.stream.shutdown(how)?)
    }

    pub fn into_bytes_stream(self) -> (Bytes, TcpStream) {
        (self.connection.into_bytes(), self.stream)
    }

    pub fn set_compression_threshold<T: Into<Option<usize>>>(&mut self, threshold: T) {
        self.connection.set_compression_threshold(threshold)
    }

    pub fn set_encryption_secret(&mut self, secret: &[u8]) {
        self.connection.set_encryption_secret(secret)
    }
}
