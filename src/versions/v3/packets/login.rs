//
// Clientbound
//

pub mod s2c {
    use crate::packet::prelude::*;

    impl_packets_enum![Disconnect, EncryptionRequest, LoginSuccess];

    #[derive(Debug, Packet, BufType)]
    #[packet(id = 0x00)]
    pub struct Disconnect {
        // JSON
        pub reason: String,
    }

    #[derive(Debug, Packet, BufType)]
    #[packet(id = 0x01)]
    pub struct EncryptionRequest {
        pub server_id: String,
        #[buftype(with = "u16_length_prefix_bytes")]
        pub public_key: Vec<u8>,
        #[buftype(with = "u16_length_prefix_bytes")]
        pub verify_token: Vec<u8>,
    }

    #[derive(Debug, Packet, BufType)]
    #[packet(id = 0x02)]
    pub struct LoginSuccess {
        pub uuid: String,
        pub username: String,
    }
}

//
// Serverbound
//

pub mod c2s {
    use crate::packet::prelude::*;

    impl_packets_enum![LoginStart, EncryptionResponse];

    #[derive(Debug, Packet, BufType)]
    #[packet(id = 0x00)]
    pub struct LoginStart {
        pub username: String,
    }

    #[derive(Debug, Packet, BufType)]
    #[packet(id = 0x01)]
    pub struct EncryptionResponse {
        #[buftype(with = "u16_length_prefix_bytes")]
        pub shared_secret: Vec<u8>,
        #[buftype(with = "u16_length_prefix_bytes")]
        pub verify_token: Vec<u8>,
    }
}
