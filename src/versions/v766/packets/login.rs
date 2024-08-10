use super::super::super::v765::packets::login as prev;

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
        LoginPluginRequest,
        CookieRequest,
    ];

    // 0x00
    pub use super::prev::s2c::Disconnect;

    #[derive(Debug, Packet, BufPacket)]
    #[packet(id = 0x01)]
    pub struct EncryptionRequest {
        pub server_id: String,
        #[packet(with = "length_prefix_bytes")]
        pub public_key: Vec<u8>,
        #[packet(with = "length_prefix_bytes")]
        pub verify_token: Vec<u8>,
        pub should_authenticate: bool,
    }

    #[derive(Debug, Packet, BufPacket)]
    #[packet(id = 0x02)]
    pub struct LoginSuccess {
        pub uuid: Uuid,
        pub username: String,
        #[packet(with = "length_prefix_array")]
        pub properties: Vec<login_success::Property>,
        pub strict_error_handling: bool,
    }

    pub use super::prev::s2c::login_success;

    // 0x03
    pub use super::prev::s2c::SetCompression;

    // 0x04
    pub use super::prev::s2c::LoginPluginRequest;

    #[derive(Debug, Packet, BufPacket)]
    #[packet(id = 0x05)]
    pub struct CookieRequest {
        pub key: Identifier,
    }
}

//
// Serverbound
//

pub mod c2s {
    use crate::packet::prelude::*;

    impl_packets_enum![
        LoginStart,
        EncryptionResponse,
        LoginPluginResponse,
        LoginAcknowledged,
        CookieResponse,
    ];

    // 0x00
    pub use super::prev::c2s::LoginStart;

    // 0x01
    pub use super::prev::c2s::EncryptionResponse;

    // 0x02
    pub use super::prev::c2s::LoginPluginResponse;

    // 0x03
    pub use super::prev::c2s::LoginAcknowledged;

    #[derive(Debug, Packet, BufPacket)]
    #[packet(id = 0x04)]
    pub struct CookieResponse {
        pub key: Identifier,
        #[packet(with = "option_length_prefix_bytes")]
        pub payload: Option<Vec<u8>>,
    }
}
