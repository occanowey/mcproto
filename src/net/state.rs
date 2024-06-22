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
        impl crate::net::state::SidedStateReadPacket<crate::net::side::Server, $state> for $packet {}
        impl crate::net::state::SidedStateWritePacket<crate::net::side::Client, $state> for $packet {}
    };

    (s2c, $state: ty, $packet: ty) => {
        impl crate::net::state::SidedStateWritePacket<crate::net::side::Server, $state> for $packet {}
        impl crate::net::state::SidedStateReadPacket<crate::net::side::Client, $state> for $packet {}
    };
}

pub(crate) use impl_sided_state_packet;

macro_rules! impl_state {
    ($state: ident ($label: expr) $(, $($side: tt [$($packet: ty),* $(,)?]),*)? $(,)?) => {
        pub struct $state;
        impl crate::net::state::NetworkState for $state {
            const LABEL: &'static str = $label;
        }

        $($($(crate::net::state::impl_sided_state_packet!($side, $state, $packet);)*)*)?
    };
}

pub(crate) use impl_state;
