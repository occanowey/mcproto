use super::super::super::v758::packets::login as prev;

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

    #[derive(Debug, Packet, BufType)]
    #[packet(id = 0x02)]
    pub struct LoginSuccess {
        pub uuid: Uuid,
        pub username: String,
        #[buftype(with = "length_prefix_array")]
        pub properties: Vec<login_success::Property>,
    }

    pub mod login_success {
        use crate::packet::prelude::*;

        #[derive(Debug, BufType)]
        pub struct Property {
            pub name: String,
            pub value: String,
            pub signature: Option<String>,
        }
    }

    // 0x03
    pub use super::prev::s2c::SetCompression;

    // 0x03
    pub use super::prev::s2c::LoginPluginRequest;
}

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
    }

    pub mod login_start {
        use crate::packet::prelude::*;

        #[derive(Debug, BufType)]
        pub struct SignatureData {
            pub timestamp: i64,
            #[buftype(with = "length_prefix_bytes")]
            pub public_key: Vec<u8>,
            #[buftype(with = "length_prefix_bytes")]
            pub signature: Vec<u8>,
        }
    }

    // 0x01
    #[derive(Debug, Packet, BufType)]
    #[packet(id = 0x01)]
    pub struct EncryptionResponse {
        #[buftype(with = "length_prefix_bytes")]
        pub shared_secret: Vec<u8>,
        pub verify_token_or_message_signature: encryption_response::VerifyTokenOrMessageSignature,
    }

    pub mod encryption_response {
        use crate::packet::prelude::*;

        #[derive(Debug)]
        pub enum VerifyTokenOrMessageSignature {
            VerifyToken(Vec<u8>),
            MessageSignature {
                salt: i64,
                message_signature: Vec<u8>,
            },
        }

        impl BufType for VerifyTokenOrMessageSignature {
            fn buf_read_len<B: bytes::Buf>(buf: &mut B) -> Result<(Self, usize), ReadError> {
                let (has_verify_token, has_verify_token_len) = bool::buf_read_len(buf)?;
                let (verify_token_or_message_signature, vtms_len) = if has_verify_token {
                    let (verify_token, verify_token_len) = length_prefix_bytes::buf_read_len(buf)?;
                    (Self::VerifyToken(verify_token), verify_token_len)
                } else {
                    let (salt, salt_len) = i64::buf_read_len(buf)?;
                    let (message_signature, message_signature_len) =
                        length_prefix_bytes::buf_read_len(buf)?;
                    (
                        Self::MessageSignature {
                            salt,
                            message_signature,
                        },
                        salt_len + message_signature_len,
                    )
                };

                Ok((
                    verify_token_or_message_signature,
                    has_verify_token_len + vtms_len,
                ))
            }

            fn buf_write<B: bytes::BufMut>(&self, buf: &mut B) {
                match self {
                    Self::VerifyToken(verify_token) => {
                        bool::buf_write(&true, buf);
                        length_prefix_bytes::buf_write(verify_token, buf);
                    }
                    Self::MessageSignature {
                        salt,
                        message_signature,
                    } => {
                        bool::buf_write(&false, buf);
                        salt.buf_write(buf);
                        length_prefix_bytes::buf_write(message_signature, buf);
                    }
                }
            }
        }
    }

    // 0x02
    pub use super::prev::c2s::LoginPluginResponse;
}
