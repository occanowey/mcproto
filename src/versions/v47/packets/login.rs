use super::super::super::v5::packets::login as prev;

//
// Clientbound
//

pub mod s2c {
    use crate::packet::prelude::*;

    impl_packets_enum![Disconnect, EncryptionRequest, LoginSuccess, SetCompression];

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
    }

    // 0x02
    pub use super::prev::s2c::LoginSuccess;

    #[derive(Debug, Packet, BufPacket)]
    #[packet(id = 0x03)]
    pub struct SetCompression {
        #[packet(with = "i32_as_v32")]
        pub threshold: i32,
    }
}

//
// Serverbound
//

pub mod c2s {
    use crate::packet::prelude::*;

    impl_packets_enum![LoginStart, EncryptionResponse];

    // 0x00
    pub use super::prev::c2s::LoginStart;

    #[derive(Debug, Packet, BufPacket)]
    #[packet(id = 0x01)]
    pub struct EncryptionResponse {
        #[packet(with = "length_prefix_bytes")]
        pub shared_secret: Vec<u8>,
        #[packet(with = "length_prefix_bytes")]
        pub verify_token: Vec<u8>,
    }
}
