use crate::{
    handshake, role,
    state::{self, impl_state},
};

use super::packets::{configuration, login, status};

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
    [ConfigurationState],
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
        login::c2s::LoginAcknowledged,
    ],
);

impl state::RoleStatePackets<role::Client> for LoginState {
    type RecvPacket = login::s2c::Packets;
}

impl state::RoleStatePackets<role::Server> for LoginState {
    type RecvPacket = login::c2s::Packets;
}

//
// Configuration State
//
impl_state!(
    ConfigurationState("configuration"),
    [],
    s2c[
        configuration::s2c::ClientboundPluginMessage,
        configuration::s2c::Disconnect,
        configuration::s2c::FinishConfiguration,
        configuration::s2c::ClientboundKeepAlive,
        configuration::s2c::Ping,
        configuration::s2c::RegistryData,
        configuration::s2c::ResourcePack,
        configuration::s2c::FeatureFlags,
        configuration::s2c::UpdateTags,
    ],
    c2s[
        configuration::c2s::ClientInformation,
        configuration::c2s::ServerboundPluginMessage,
        configuration::c2s::FinishConfiguration,
        configuration::c2s::ServerboundKeepAlive,
        configuration::c2s::Pong,
        configuration::c2s::ResourcePackResponse,
    ],
);

impl state::RoleStatePackets<role::Client> for ConfigurationState {
    type RecvPacket = configuration::s2c::Packets;
}

impl state::RoleStatePackets<role::Server> for ConfigurationState {
    type RecvPacket = configuration::c2s::Packets;
}
