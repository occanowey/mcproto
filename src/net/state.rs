use crate::{
    net::side::{Client, Server},
    packet::{
        handshaking::Handshake,
        status::*,
        login::{self, *},
        play,
    },
};

pub trait NetworkState {}

mod sealed {
    use crate::{packet::{PacketRead, PacketWrite}, net::side::NetworkSide};
    use super::NetworkState;

    pub trait SidedStateReadPacket<Side: NetworkSide, State: NetworkState>: PacketRead {}
    pub trait SidedStateWritePacket<Side: NetworkSide, State: NetworkState>: PacketWrite {}
}

pub(crate) use self::sealed::*;
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

impl_sided_state_packet!(s2c, LoginState, login::Disconnect);
impl_sided_state_packet!(s2c, LoginState, EncryptionRequest);
impl_sided_state_packet!(s2c, LoginState, LoginSuccess);
impl_sided_state_packet!(s2c, LoginState, SetCompression);
impl_sided_state_packet!(s2c, LoginState, LoginPluginRequest);

impl_sided_state_packet!(c2s, LoginState, LoginStart);
impl_sided_state_packet!(c2s, LoginState, EncryptionResponse);
impl_sided_state_packet!(c2s, LoginState, LoginPluginResponse);

//
// Play State
//
pub struct PlayState;
impl NetworkState for PlayState {}

impl_sided_state_packet!(s2c, PlayState, play::Disconnect);
