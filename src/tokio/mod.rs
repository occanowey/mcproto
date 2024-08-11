use std::convert::TryInto;

use bytes::Bytes;

use crate::{connection, error, packet, role, state};

use tokio::{
    io::{AsyncReadExt as _, AsyncWriteExt as _},
    net::{TcpStream, ToSocketAddrs},
};
use tracing::{trace_span, Instrument};

pub struct TokioConnection<Role, State>
where
    Role: role::ConnectionRole,
    State: state::ProtocolState,
{
    stream: TcpStream,
    connection: connection::Connection<Role, State>,
}

pub fn accept_tokio_stream<Role: role::ConnectionRole, State: state::ProtocolState>(
    stream: TcpStream,
) -> Result<TokioConnection<Role, State>, error::Error> {
    stream.set_nodelay(true)?;

    Ok(TokioConnection {
        stream,
        connection: connection::create_connection(),
    })
}

pub async fn connect_tokio_stream<
    Addr: ToSocketAddrs,
    Role: role::ConnectionRole,
    State: state::ProtocolState,
>(
    addr: Addr,
) -> Result<TokioConnection<Role, State>, error::Error> {
    accept_tokio_stream(TcpStream::connect(addr).await?)
}

impl<Role, State> TokioConnection<Role, State>
where
    Role: role::ConnectionRole,
    State: state::ProtocolState,
{
    // TODO: timeout
    pub async fn next_packet(&mut self) -> Result<State::RecvPacket, error::Error>
    where
        State: state::RoleStatePackets<Role>,
        State::RecvPacket: packet::PacketFromIdBody,
    {
        loop {
            let event =
                trace_span!("connection.next_event").in_scope(|| self.connection.next_event())?;
            match event {
                connection::Event::NeedMoreData => {
                    let mut data = [0; 512];
                    let len = self
                        .stream
                        .read(&mut data)
                        .instrument(trace_span!("connection.stream.read"))
                        .await?;
                    if len == 0 {
                        return Err(error::Error::StreamShutdown);
                    }
                    trace_span!("connection.recieve_data")
                        .in_scope(|| self.connection.recieve_data(&data[0..len]));
                }
                connection::Event::Packet(packet) => return Ok(packet),
            }
        }
    }

    pub async fn expect_next_packet<Packet>(&mut self) -> Result<Packet, error::Error>
    where
        Packet: state::RoleStateReadPacket<Role, State>,

        State: state::RoleStatePackets<Role>,
        State::RecvPacket: packet::PacketFromIdBody,
        State::RecvPacket: TryInto<Packet, Error = error::Error>,
    {
        self.next_packet().await?.try_into()
    }

    pub async fn write_packet<Packet: state::RoleStateWritePacket<Role, State>>(
        &mut self,
        packet: Packet,
    ) -> Result<(), error::Error> {
        let data = trace_span!("connection.send").in_scope(|| self.connection.send(packet));

        Ok(self
            .stream
            .write_all(&data)
            .instrument(trace_span!("connection.stream.write_all"))
            .await?)
    }

    pub async fn write_id_body(&mut self, id: i32, body: &mut Bytes) -> Result<(), error::Error> {
        Ok(self
            .stream
            .write_all(&self.connection.send_id_body(id, body))
            .await?)
    }

    pub fn next_state<NextState>(self) -> TokioConnection<Role, NextState>
    where
        NextState: state::ProtocolState,
        NextState: state::NextProtocolState<State>,
    {
        TokioConnection {
            stream: self.stream,
            connection: self.connection.next_state(),
        }
    }

    pub async fn shutdown(&mut self) -> Result<(), error::Error> {
        Ok(self.stream.shutdown().await?)
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
