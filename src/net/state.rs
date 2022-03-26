use crate::packet::{handshaking::Handshake, login::*, status::*};

pub(crate) use self::sealed::*;

mod sealed {
    use crate::packet::{PacketRead, PacketWrite};

    pub trait NetworkState {}

    pub trait NetworkSide {}

    pub trait SidedStateReadPacket<Side, State>: PacketRead {}
    pub trait SidedStateWritePacket<Side, State>: PacketWrite {}
}

pub struct Server;
impl NetworkSide for Server {}

pub struct Client;
impl NetworkSide for Client {}

macro_rules! impl_sided_state_packet {
    (c2s, $state: ty, $packet: ty) => {
        impl SidedStateReadPacket<Server, $state> for $packet {}
        impl SidedStateWritePacket<Client, $state> for $packet {}
    };

    (s2c, $state: ty, $packet: ty) => {
        impl SidedStateWritePacket<Server, $state> for $packet {}
        impl SidedStateReadPacket<Client, $state> for $packet {}
    };
}

//
// Handshaking State
//
pub struct Handshaking;
impl NetworkState for Handshaking {}

impl_sided_state_packet!(c2s, Handshaking, Handshake);

//
// Status State
//
pub struct StatusState;
impl NetworkState for StatusState {}

impl_sided_state_packet!(s2c, StatusState, Response);
impl_sided_state_packet!(s2c, StatusState, Pong);

impl_sided_state_packet!(c2s, StatusState, Request);
impl_sided_state_packet!(c2s, StatusState, Ping);

//
// Login State
//
pub struct LoginState;
impl NetworkState for LoginState {}

impl_sided_state_packet!(s2c, LoginState, Disconnect);

impl_sided_state_packet!(c2s, LoginState, LoginStart);
