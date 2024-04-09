use crate::{
    net::side::{Client, Server},
    packet::{
        handshaking::Handshake,
        login::{self, *},
        play,
        status::*,
    },
};

pub trait NetworkState {
    const LABEL: &'static str;
}

mod sealed {
    use super::NetworkState;
    use crate::{
        net::side::NetworkSide,
        packet::{PacketRead, PacketWrite},
    };

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
pub struct HandshakingState;
impl NetworkState for HandshakingState {
    const LABEL: &'static str = "handshaking";
}

impl_sided_state_packet!(c2s, HandshakingState, Handshake);

//
// Status State
//
pub struct StatusState;
impl NetworkState for StatusState {
    const LABEL: &'static str = "status";
}

impl_sided_state_packet!(s2c, StatusState, Response);
impl_sided_state_packet!(s2c, StatusState, Pong);

impl_sided_state_packet!(c2s, StatusState, Request);
impl_sided_state_packet!(c2s, StatusState, Ping);

//
// Login State
//
pub struct LoginState;
impl NetworkState for LoginState {
    const LABEL: &'static str = "login";
}

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
impl NetworkState for PlayState {
    const LABEL: &'static str = "play";
}

impl_sided_state_packet!(s2c, PlayState, play::Disconnect);
