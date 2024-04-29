use crate::{
    net::side::{Client, Server},
    packet::{configuration, handshaking, login, play, status},
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

macro_rules! impl_state {
    ($state: ident ($label: expr) $(, $($side: tt [$($packet: ty),* $(,)?]),*)? $(,)?) => {
        pub struct $state;
        impl NetworkState for $state {
            const LABEL: &'static str = $label;
        }

        $($($(impl_sided_state_packet!($side, $state, $packet);)*)*)?
    };
}

//
// Handshaking State
//
impl_state!(HandshakingState("handshaking"), c2s[handshaking::Handshake]);

//
// Status State
//
impl_state!(
    StatusState("status"),
    s2c[status::StatusResponse, status::PingResponse],
    c2s[status::StatusRequest, status::PingRequest],
);

//
// Login State
//
impl_state!(
    LoginState("login"),
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
impl_state!(PlayState("play"), s2c[play::Disconnect]);
