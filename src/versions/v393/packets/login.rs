use super::super::super::v340::packets::login as prev;

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

    // 0x02
    pub use super::prev::s2c::LoginSuccess;

    // 0x03
    pub use super::prev::s2c::SetCompression;

    #[derive(Debug, Packet, BufType)]
    #[packet(id = 0x04)]
    pub struct LoginPluginRequest {
        pub message_id: i32,
        pub channel: Identifier,
        #[buftype(with = "remaining_bytes")]
        pub data: Vec<u8>,
    }
}

//
// Serverbound
//

pub mod c2s {
    use crate::packet::prelude::*;

    impl_packets_enum![LoginStart, EncryptionResponse, LoginPluginResponse];

    // 0x00
    pub use super::prev::c2s::LoginStart;

    // 0x01
    pub use super::prev::c2s::EncryptionResponse;

    #[derive(Debug, Packet, BufType)]
    #[packet(id = 0x02)]
    pub struct LoginPluginResponse {
        #[buftype(with = "i32_as_v32")]
        pub message_id: i32,
        pub successful: bool,
        #[buftype(with = "remaining_bytes")]
        pub data: Vec<u8>,
    }
}
