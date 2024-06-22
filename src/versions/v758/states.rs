use crate::{
    handshake,
    state::{self, impl_state},
};

use super::packets::{configuration, login, play, status};

//
// Handshaking State
//
impl state::NextHandlerState<handshake::HandshakingState> for StatusState {}
impl state::NextHandlerState<handshake::HandshakingState> for LoginState {}

//
// Status State
//
impl_state!(
    StatusState("status"),
    [],
    s2c[status::StatusResponse, status::PingResponse],
    c2s[status::StatusRequest, status::PingRequest],
);

//
// Login State
//
impl_state!(
    LoginState("login"),
    [ConfigurationState],
    s2c[
        login::Disconnect,
        login::EncryptionRequest,
        login::LoginSuccess,
        login::SetCompression,
        login::LoginPluginRequest,
    ],
    c2s[
        login::LoginStart,
        login::EncryptionResponse,
        login::LoginPluginResponse,
        login::LoginAcknowledged,
    ],
);

//
// Configuration State
//
impl_state!(
    ConfigurationState("configuration"),
    [PlayState],
    s2c[
        configuration::ClientboundPluginMessage,
        configuration::Disconnect,
        configuration::FinishConfiguration,
        configuration::ClientboundKeepAlive,
        configuration::Ping,
        configuration::RegistryData,
        configuration::RemoveResourcePack,
        configuration::AddResourcePack,
        configuration::FeatureFlags,
        configuration::UpdateTags,
    ],
    c2s[
        configuration::ClientInformation,
        configuration::ServerboundPluginMessage,
        configuration::AcknowledgeFinishConfiguration,
        configuration::ServerboundKeepAlive,
        configuration::Pong,
        configuration::ResourcePackResponse,
    ],
);

//
// Play State
//
impl_state!(PlayState("play"), [], s2c[play::Disconnect]);
