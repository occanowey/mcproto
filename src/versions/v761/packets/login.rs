use super::super::super::v760::packets::login as prev;

//
// Clientbound
//

pub use prev::s2c;

//
// Serverbound
//

pub mod c2s {
    use crate::packet::prelude::*;

    impl_packets_enum![LoginStart, EncryptionResponse, LoginPluginResponse];

    #[derive(Debug, Packet, BufType)]
    #[packet(id = 0x00)]
    pub struct LoginStart {
        pub username: String,
        pub uuid: Option<Uuid>,
    }

    // 0x01
    #[derive(Debug, Packet, BufType)]
    #[packet(id = 0x01)]
    pub struct EncryptionResponse {
        #[buftype(with = "length_prefix_bytes")]
        pub shared_secret: Vec<u8>,
        #[buftype(with = "length_prefix_bytes")]
        pub verify_token: Vec<u8>,
    }

    // 0x02
    pub use super::prev::c2s::LoginPluginResponse;
}
