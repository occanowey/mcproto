use super::{impl_packet_enum, Packet, PacketBuilder, PacketRead, PacketWrite};
use crate::{
    error::Result,
    types::{
        proxy::{i32_as_v32, length_prefix_bytes},
        v32, Identifier, McRead,
    },
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
    #[packet(with = "length_prefix_bytes")]
    pub public_key: Vec<u8>,
    #[packet(with = "length_prefix_bytes")]
    pub verify_token: Vec<u8>,
}

#[derive(Debug, Packet)]
#[packet(id = 0x02)]
pub struct LoginSuccess {
    pub uuid: Uuid,
    pub username: String,
    pub properties: Vec<login_success::Property>,
}

mod login_success {
    use crate::types::{proxy::bool_option, McRead, McWrite};

    #[derive(Debug)]
    pub struct Property {
        pub name: String,
        pub value: String,
        pub signature: Option<String>,
    }

    impl McRead for Property {
        fn read<R: std::io::prelude::Read>(reader: &mut R) -> std::io::Result<(Self, usize)>
        where
            Self: std::marker::Sized,
        {
            let (name, name_len) = String::read(reader)?;
            let (value, value_len) = String::read(reader)?;
            let (signature, signature_len) = bool_option::mc_read(reader)?;

            let property = Property {
                name,
                value,
                signature,
            };

            Ok((property, name_len + value_len + signature_len))
        }
    }

    impl McWrite for Property {
        fn write<W: std::io::prelude::Write>(&self, writer: &mut W) -> std::io::Result<()> {
            self.name.write(writer)?;
            self.value.write(writer)?;
            bool_option::mc_write(writer, &self.signature)?;

            Ok(())
        }
    }
}

impl PacketRead for LoginSuccess {
    fn read_data<R: Read>(reader: &mut R, _data_length: usize) -> Result<Self> {
        let (uuid, _) = Uuid::read(reader)?;
        let (username, _) = String::read(reader)?;

        let mut properties = Vec::new();
        let (properties_count, _) = v32::read(reader)?;
        for _ in 0..properties_count.0 {
            let (property, _) = login_success::Property::read(reader)?;
            properties.push(property);
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
            packet.write(property)?;
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
    #[packet(with = "length_prefix_bytes")]
    pub shared_secret: Vec<u8>,
    #[packet(with = "length_prefix_bytes")]
    pub verify_token: Vec<u8>,
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
