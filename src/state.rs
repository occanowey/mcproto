use crate::{
    packet::{PacketRead, PacketWrite},
    role::ConnectionRole,
};

pub trait ProtocolState {
    const LABEL: &'static str;
}

pub trait NextProtocolState<Current: ProtocolState>: ProtocolState {}

pub trait RoleStateReadPacket<Role: ConnectionRole, State: ProtocolState>: PacketRead {}
pub trait RoleStateWritePacket<Role: ConnectionRole, State: ProtocolState>: PacketWrite {}

macro_rules! impl_role_state_packet {
    (c2s, $state: ty, $packet: ty) => {
        impl crate::state::RoleStateReadPacket<crate::role::Server, $state> for $packet {}
        impl crate::state::RoleStateWritePacket<crate::role::Client, $state> for $packet {}
    };

    (s2c, $state: ty, $packet: ty) => {
        impl crate::state::RoleStateWritePacket<crate::role::Server, $state> for $packet {}
        impl crate::state::RoleStateReadPacket<crate::role::Client, $state> for $packet {}
    };
}

pub(crate) use impl_role_state_packet;

macro_rules! impl_state {
    ($state: ident ($label: expr), [$($next_state: ty), *] $(, $($role: tt [$($packet: ty),* $(,)?]),*)? $(,)?) => {
        pub struct $state;

        impl crate::state::ProtocolState for $state {
            const LABEL: &'static str = $label;
        }

        $(impl crate::state::NextProtocolState<$state> for $next_state {})*

        $($($(crate::state::impl_role_state_packet!($role, $state, $packet);)*)*)?
    };
}

pub(crate) use impl_state;
