use super::super::super::v759::packets::login as prev;

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
        pub signature_data: Option<login_start::SignatureData>,
        pub uuid: Option<Uuid>,
    }

    pub use super::prev::c2s::login_start;

    // 0x01
    pub use super::prev::c2s::EncryptionResponse;

    pub use super::prev::c2s::encryption_response;

    // 0x02
    pub use super::prev::c2s::LoginPluginResponse;
}
