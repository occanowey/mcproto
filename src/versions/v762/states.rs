use crate::{
    handshake, role,
    state::{self, impl_state},
};

use super::packets::{login, status};

//
// Handshaking State
//
impl state::NextProtocolState<handshake::HandshakingState> for StatusState {}
impl state::NextProtocolState<handshake::HandshakingState> for LoginState {}

//
// Status State
//
impl_state!(
    StatusState("status"),
    [],
    s2c[status::s2c::StatusResponse, status::s2c::PingResponse],
    c2s[status::c2s::StatusRequest, status::c2s::PingRequest],
);

impl state::RoleStatePackets<role::Client> for StatusState {
    type RecvPacket = status::s2c::Packets;
}

impl state::RoleStatePackets<role::Server> for StatusState {
    type RecvPacket = status::c2s::Packets;
}

//
// Login State
//
impl_state!(
    LoginState("login"),
    [],
    s2c[
        login::s2c::Disconnect,
        login::s2c::EncryptionRequest,
        login::s2c::LoginSuccess,
        login::s2c::SetCompression,
        login::s2c::LoginPluginRequest,
    ],
    c2s[
        login::c2s::LoginStart,
        login::c2s::EncryptionResponse,
        login::c2s::LoginPluginResponse,
    ],
);

impl state::RoleStatePackets<role::Client> for LoginState {
    type RecvPacket = login::s2c::Packets;
}

impl state::RoleStatePackets<role::Server> for LoginState {
    type RecvPacket = login::c2s::Packets;
}
