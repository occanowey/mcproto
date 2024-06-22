use super::super::super::v763::packets::login as prev;

//
// Clientbound
//

pub use prev::s2c;

//
// Serverbound
//

pub mod c2s {
    use crate::packet::prelude::*;

    impl_packets_enum![
        LoginStart,
        EncryptionResponse,
        LoginPluginResponse,
        LoginAcknowledged
    ];

    #[derive(Debug, Packet, PacketRead, PacketWrite)]
    #[packet(id = 0x00)]
    pub struct LoginStart {
        pub username: String,
        pub uuid: Uuid,
    }

    // 0x01
    pub use super::prev::c2s::EncryptionResponse;

    // 0x02
    pub use super::prev::c2s::LoginPluginResponse;

    #[derive(Debug, Packet, PacketRead, PacketWrite)]
    #[packet(id = 0x03)]
    pub struct LoginAcknowledged;
}
