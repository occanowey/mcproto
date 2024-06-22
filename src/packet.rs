use crate::types::ReadError;

use bytes::{Buf, BufMut};

macro_rules! impl_packet_enum {
    ($side:ident {$($id:literal => $packet:ident),* $(,)?}) => {
        pub mod $side {
            #[derive(Debug)]
            pub enum Packet {
                $($packet(super::$packet),)*

                Unknown(i32)
            }

            #[automatically_derived]
            impl Packet {
                pub fn is_known(&self) -> bool {
                    !matches!(self, Self::Unknown(_))
                }

                pub fn from_id_body<B: bytes::Buf>(id: i32, data: &mut B) -> std::result::Result<Self, crate::types::ReadError> {
                    match id {
                        $($id => <super::$packet as crate::packet::PacketRead>::read_body(data).map(Self::$packet),)*

                        other => Ok(Self::Unknown(other)),
                    }
                }
            }
        }
    };
}

pub(crate) use impl_packet_enum;

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
