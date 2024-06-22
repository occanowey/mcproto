use crate::types::ReadError;
use bytes::{Buf, BufMut, Bytes};

pub trait Packet {
    const PACKET_ID: i32;
}

pub trait PacketRead: Packet + Sized {
    /// Read fields after length & id
    fn read_body<B: Buf>(data: &mut B) -> std::result::Result<Self, ReadError>;
}

pub trait PacketWrite: Packet {
    fn write_body<B: BufMut>(&self, buf: &mut B);
}

pub trait PacketFromIdBody {
    fn from_id_body(id: i32, body: Bytes) -> std::result::Result<Self, ReadError>
    where
        Self: Sized;
}

macro_rules! impl_packets_enum {
    [$($packet:ident),* $(,)?] => {
        #[derive(Debug)]
        pub enum Packets {
            $($packet($packet),)*

            Unknown(i32, bytes::Bytes)
        }

        #[automatically_derived]
        impl Packets {
            pub fn is_known(&self) -> bool {
                !matches!(self, Self::Unknown(_, _))
            }
        }

        #[automatically_derived]
        impl crate::packet::PacketFromIdBody for Packets {
            fn from_id_body(id: i32, mut body: bytes::Bytes) -> std::result::Result<Self, crate::types::ReadError> {
                match id {
                    $(<$packet as crate::packet::Packet>::PACKET_ID => <$packet as crate::packet::PacketRead>::read_body(&mut body).map(Self::$packet),)*

                    other => Ok(Self::Unknown(other, body)),
                }
            }
        }

        $(
            impl std::convert::TryFrom<Packets> for $packet {
                type Error = crate::error::Error;

                fn try_from(value: Packets) -> Result<Self, Self::Error> {
                    match value {
                        Packets::$packet(packet) => Ok(packet),
                        _ => Err(crate::error::Error::IncorectPacket),
                    }
                }
            }
        )*
    };
}

pub(crate) use impl_packets_enum;

#[allow(unused_imports)]
pub(crate) mod prelude {
    pub use super::{Packet, PacketRead, PacketWrite};
    pub use crate::types::{
        proxy::{
            i32_as_v32, length_prefix_array, length_prefix_bytes, option_length_prefix_bytes,
            remaining_bytes, u16_length_prefix_bytes,
        },
        BufType, Identifier, ReadError,
    };
    pub use bytes::{Buf, BufMut};
    pub use packet_derive::{Packet, PacketRead, PacketWrite};
    pub use std::collections::HashMap;
    pub use uuid::Uuid;

    pub(crate) use super::impl_packets_enum;
    pub(crate) use crate::types::v32_prefix_enum;
}
