use crate::{
    handshake,
    state::{self, impl_state},
};

use super::packets::{configuration, login, play, status};

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

//
// Configuration State
//
impl_state!(
    ConfigurationState("configuration"),
    [PlayState],
    s2c[
        configuration::s2c::ClientboundPluginMessage,
        configuration::s2c::Disconnect,
        configuration::s2c::FinishConfiguration,
        configuration::s2c::ClientboundKeepAlive,
        configuration::s2c::Ping,
        configuration::s2c::RegistryData,
        configuration::s2c::RemoveResourcePack,
        configuration::s2c::AddResourcePack,
        configuration::s2c::FeatureFlags,
        configuration::s2c::UpdateTags,
    ],
    c2s[
        configuration::c2s::ClientInformation,
        configuration::c2s::ServerboundPluginMessage,
        configuration::c2s::AcknowledgeFinishConfiguration,
        configuration::c2s::ServerboundKeepAlive,
        configuration::c2s::Pong,
        configuration::c2s::ResourcePackResponse,
    ],
);

//
// Play State
//
impl_state!(PlayState("play"), [], s2c[play::s2c::Disconnect]);
