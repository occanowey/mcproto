use super::super::super::v578::packets::login as prev;

//
// Clientbound
//

pub mod s2c {
    use crate::packet::prelude::*;

    impl_packets_enum![
        Disconnect,
        EncryptionRequest,
        LoginSuccess,
        SetCompression,
        LoginPluginRequest
    ];

    // 0x00
    pub use super::prev::s2c::Disconnect;

    // 0x01
    pub use super::prev::s2c::EncryptionRequest;

    #[derive(Debug, Packet, PacketRead, PacketWrite)]
    #[packet(id = 0x02)]
    pub struct LoginSuccess {
        pub uuid: Uuid,
        pub username: String,
    }

    // 0x03
    pub use super::prev::s2c::SetCompression;

    // 0x03
    pub use super::prev::s2c::LoginPluginRequest;
}

//
// Serverbound
//

pub use prev::c2s;
