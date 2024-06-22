pub trait NetworkState {
    const LABEL: &'static str;
}

mod sealed {
    use super::NetworkState;
    use crate::{
        packet::{PacketRead, PacketWrite},
        side::NetworkSide,
    };

    pub trait SidedStateReadPacket<Side: NetworkSide, State: NetworkState>: PacketRead {}
    pub trait SidedStateWritePacket<Side: NetworkSide, State: NetworkState>: PacketWrite {}

    pub trait NextHandlerState<Current: NetworkState>: NetworkState {}
}

pub(crate) use self::sealed::*;

macro_rules! impl_sided_state_packet {
    (c2s, $state: ty, $packet: ty) => {
        impl crate::state::SidedStateReadPacket<crate::side::Server, $state> for $packet {}
        impl crate::state::SidedStateWritePacket<crate::side::Client, $state> for $packet {}
    };

    (s2c, $state: ty, $packet: ty) => {
        impl crate::state::SidedStateWritePacket<crate::side::Server, $state> for $packet {}
        impl crate::state::SidedStateReadPacket<crate::side::Client, $state> for $packet {}
    };
}

pub(crate) use impl_sided_state_packet;

macro_rules! impl_state {
    ($state: ident ($label: expr), [$($next_state: ty), *] $(, $($side: tt [$($packet: ty),* $(,)?]),*)? $(,)?) => {
        pub struct $state;
        impl crate::state::NetworkState for $state {
            const LABEL: &'static str = $label;
        }

        $(impl crate::state::NextHandlerState<$state> for $next_state {})*

        $($($(crate::state::impl_sided_state_packet!($side, $state, $packet);)*)*)?
    };
}

pub(crate) use impl_state;
