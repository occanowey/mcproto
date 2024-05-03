use super::{impl_packet_enum, Packet, PacketBuilder, PacketRead, PacketWrite};
use crate::{
    error::Result,
    types::{proxy::i32_as_v32, v32, Identifier, LengthPrefixByteArray, McRead},
    ReadExt,
};
use packet_derive::{Packet, PacketRead, PacketWrite};
use std::io::Read;
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
    pub public_key: LengthPrefixByteArray,
    pub verify_token: LengthPrefixByteArray,
}

#[derive(Debug, Packet)]
#[packet(id = 0x02)]
pub struct LoginSuccess {
    pub uuid: Uuid,
    pub username: String,
    pub properties: Vec<login_success::Property>,
}

mod login_success {
    #[derive(Debug)]
    pub struct Property {
        pub name: String,
        pub value: String,
        pub signature: Option<String>,
    }
}

impl PacketRead for LoginSuccess {
    fn read_data<R: Read>(reader: &mut R, _data_length: usize) -> Result<Self> {
        let (uuid, _) = Uuid::read(reader)?;
        let (username, _) = String::read(reader)?;

        let mut properties = Vec::new();
        let (properties_count, _) = v32::read(reader)?;
        for _ in 0..properties_count.0 {
            let (name, _) = String::read(reader)?;
            let (value, _) = String::read(reader)?;

            let (has_signature, _) = bool::read(reader)?;

            let signature = if has_signature {
                let (signature, _) = String::read(reader)?;
                Some(signature)
            } else {
                None
            };

            properties.push(login_success::Property {
                name,
                value,
                signature,
            });
        }

        Ok(LoginSuccess {
            uuid,
            username,
            properties,
        })
    }
}

impl PacketWrite for LoginSuccess {
    fn write_data(&self, packet: &mut PacketBuilder) -> Result<()> {
        packet.write(&self.uuid)?;
        packet.write(&self.username)?;

        let properties_count = self.properties.len();
        // TODO: return error rather than panic
        assert!(properties_count < (v32::MAX as usize));
        packet.write(&v32(properties_count as _))?;

        for property in &self.properties {
            packet.write(&property.name)?;
            packet.write(&property.value)?;

            if let Some(signature) = &property.signature {
                packet.write(&true)?;
                packet.write(signature)?;
            } else {
                packet.write(&false)?;
            }
        }

        Ok(())
    }
}

#[derive(Debug, Packet, PacketRead, PacketWrite)]
#[packet(id = 0x03)]
pub struct SetCompression {
    #[packet(with = "i32_as_v32")]
    pub threshold: i32,
}

#[derive(Debug, Packet)]
#[packet(id = 0x04)]
pub struct LoginPluginRequest {
    pub message_id: i32,
    pub channel: Identifier,
    pub data: Vec<u8>,
}

impl PacketRead for LoginPluginRequest {
    fn read_data<R: Read>(reader: &mut R, data_length: usize) -> Result<Self> {
        let (message_id, message_id_length) = i32_as_v32::read(reader, data_length)?;
        let (channel, channel_length) = Identifier::read(reader)?;

        let data = reader.read_byte_array(data_length - message_id_length - channel_length)?;

        Ok(LoginPluginRequest {
            message_id,
            channel,
            data,
        })
    }
}

impl PacketWrite for LoginPluginRequest {
    fn write_data(&self, packet: &mut PacketBuilder) -> Result<()> {
        i32_as_v32::write(packet, &self.message_id)?;
        packet.write(&self.channel)?;

        Ok(packet.write_byte_array(&self.data)?)
    }
}

//
// Serverbound
//

impl_packet_enum!(c2s {
    0x00 => LoginStart,
    0x01 => EncryptionResponse,
    0x02 => LoginPluginResponse,
    0x02 => LoginAcknowledged,
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
    pub shared_secret: LengthPrefixByteArray,
    pub verify_token: LengthPrefixByteArray,
}

#[derive(Debug, Packet)]
#[packet(id = 0x02)]
pub struct LoginPluginResponse {
    pub message_id: i32,
    pub successful: bool,
    pub data: Vec<u8>,
}

impl PacketRead for LoginPluginResponse {
    fn read_data<R: Read>(reader: &mut R, data_len: usize) -> Result<Self> {
        let (message_id, message_id_len) = i32_as_v32::read(reader, data_len)?;
        let (successful, successful_len) = bool::read(reader)?;

        let data = reader.read_byte_array(data_len - message_id_len - successful_len)?;

        Ok(LoginPluginResponse {
            message_id,
            successful,
            data,
        })
    }
}

impl PacketWrite for LoginPluginResponse {
    fn write_data(&self, packet: &mut PacketBuilder) -> Result<()> {
        i32_as_v32::write(packet, &self.message_id)?;
        packet.write(&self.successful)?;

        Ok(packet.write_byte_array(&self.data)?)
    }
}

#[derive(Debug, Packet, PacketRead, PacketWrite)]
#[packet(id = 0x03)]
pub struct LoginAcknowledged;
