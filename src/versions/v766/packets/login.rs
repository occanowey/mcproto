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

    #[derive(Debug, Packet, PacketRead, PacketWrite)]
    #[packet(id = 0x00)]
    pub struct Disconnect {
        // Text Component (JSON)
        pub reason: String,
    }

    #[derive(Debug, Packet, PacketRead, PacketWrite)]
    #[packet(id = 0x01)]
    pub struct EncryptionRequest {
        pub server_id: String,
        #[packet(with = "length_prefix_bytes")]
        pub public_key: Vec<u8>,
        #[packet(with = "length_prefix_bytes")]
        pub verify_token: Vec<u8>,
        pub should_authenticate: bool,
    }

    #[derive(Debug, Packet, PacketRead, PacketWrite)]
    #[packet(id = 0x02)]
    pub struct LoginSuccess {
        pub uuid: Uuid,
        pub username: String,
        #[packet(with = "length_prefix_array")]
        pub properties: Vec<login_success::Property>,
        pub strict_error_handling: bool,
    }

    mod login_success {
        use crate::types::BufType;
        use crate::types::ReadError;

        #[derive(Debug)]
        pub struct Property {
            pub name: String,
            pub value: String,
            pub signature: Option<String>,
        }

        impl BufType for Property {
            fn buf_read_len<B: bytes::Buf>(buf: &mut B) -> Result<(Self, usize), ReadError> {
                let (name, name_len) = String::buf_read_len(buf)?;
                let (value, value_len) = String::buf_read_len(buf)?;
                let (signature, signature_len) = Option::buf_read_len(buf)?;

                let property = Property {
                    name,
                    value,
                    signature,
                };

                Ok((property, name_len + value_len + signature_len))
            }

            fn buf_write<B: bytes::BufMut>(&self, buf: &mut B) {
                self.name.buf_write(buf);
                self.value.buf_write(buf);
                self.signature.buf_write(buf);
            }
        }
    }

    #[derive(Debug, Packet, PacketRead, PacketWrite)]
    #[packet(id = 0x03)]
    pub struct SetCompression {
        #[packet(with = "i32_as_v32")]
        pub threshold: i32,
    }

    #[derive(Debug, Packet, PacketRead, PacketWrite)]
    #[packet(id = 0x04)]
    pub struct LoginPluginRequest {
        pub message_id: i32,
        pub channel: Identifier,
        #[packet(with = "remaining_bytes")]
        pub data: Vec<u8>,
    }

    #[derive(Debug, Packet, PacketRead, PacketWrite)]
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

    #[derive(Debug, Packet, PacketRead, PacketWrite)]
    #[packet(id = 0x00)]
    pub struct LoginStart {
        pub username: String,
        pub uuid: Uuid,
    }

    #[derive(Debug, Packet, PacketRead, PacketWrite)]
    #[packet(id = 0x01)]
    pub struct EncryptionResponse {
        #[packet(with = "length_prefix_bytes")]
        pub shared_secret: Vec<u8>,
        #[packet(with = "length_prefix_bytes")]
        pub verify_token: Vec<u8>,
    }

    #[derive(Debug, Packet, PacketRead, PacketWrite)]
    #[packet(id = 0x02)]
    pub struct LoginPluginResponse {
        #[packet(with = "i32_as_v32")]
        pub message_id: i32,
        pub successful: bool,
        #[packet(with = "remaining_bytes")]
        pub data: Vec<u8>,
    }

    #[derive(Debug, Packet, PacketRead, PacketWrite)]
    #[packet(id = 0x03)]
    pub struct LoginAcknowledged;

    #[derive(Debug, Packet, PacketRead, PacketWrite)]
    #[packet(id = 0x04)]
    pub struct CookieResponse {
        pub key: Identifier,
        #[packet(with = "option_length_prefix_bytes")]
        pub payload: Option<Vec<u8>>,
    }
}
