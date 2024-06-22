use crate::packet::{impl_packet_enum, Packet, PacketRead, PacketWrite};
use crate::types::{
    proxy::{i32_as_v32, length_prefix_array, length_prefix_bytes, remaining_bytes},
    Identifier,
};
use packet_derive::{Packet, PacketRead, PacketWrite};
use uuid::Uuid;

//
// Clientbound
//

impl_packet_enum!(s2c {
    0x00 => Disconnect,
    0x01 => EncryptionRequest,
    0x02 => LoginSuccess,
    0x03 => SetCompression,
    0x04 => LoginPluginRequest,
});

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
}

#[derive(Debug, Packet, PacketRead, PacketWrite)]
#[packet(id = 0x02)]
pub struct LoginSuccess {
    pub uuid: Uuid,
    pub username: String,
    #[packet(with = "length_prefix_array")]
    pub properties: Vec<login_success::Property>,
}

mod login_success {
    use crate::types::ReadError;
    use crate::types::{proxy::bool_option, BufType};

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
            let (signature, signature_len) = bool_option::buf_read_len(buf)?;

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
            bool_option::buf_write(&self.signature, buf);
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

//
// Serverbound
//

impl_packet_enum!(c2s {
    0x00 => LoginStart,
    0x01 => EncryptionResponse,
    0x02 => LoginPluginResponse,
    0x03 => LoginAcknowledged,
});

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
    pub message_id: i32,
    pub successful: bool,
    #[packet(with = "remaining_bytes")]
    pub data: Vec<u8>,
}

#[derive(Debug, Packet, PacketRead, PacketWrite)]
#[packet(id = 0x03)]
pub struct LoginAcknowledged;
