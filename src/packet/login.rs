use super::{impl_packet_enum, Packet, PacketBuilder, PacketRead, PacketWrite};
use crate::{
    types::{v32, Identifier, LengthPrefixByteArray, McRead},
    ReadExt,
};
use packet_derive::{Packet, PacketRead, PacketWrite};
use uuid::Uuid;
use std::io::{Read, Result};

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
#[id(0x00)]
pub struct Disconnect {
    pub reason: String,
}

#[derive(Debug, Packet, PacketRead, PacketWrite)]
#[id(0x01)]
pub struct EncryptionRequest {
    pub server_id: String,
    pub public_key: LengthPrefixByteArray,
    pub verify_token: LengthPrefixByteArray,
}

#[derive(Debug, Packet, PacketRead, PacketWrite)]
#[id(0x02)]
pub struct LoginSuccess {
    pub uuid: Uuid,
    pub username: String,
}

#[derive(Debug, Packet, PacketRead, PacketWrite)]
#[id(0x03)]
pub struct SetCompression {
    pub threshold: v32,
}

#[derive(Debug, Packet)]
#[id(0x04)]
pub struct LoginPluginRequest {
    pub message_id: v32,
    pub channel: Identifier,
    pub data: Vec<u8>,
}

impl PacketRead for LoginPluginRequest {
    fn read_data<R: Read>(reader: &mut R, data_length: usize) -> Result<Self> {
        let (message_id, message_id_length) = v32::read(reader)?;
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
        packet.write(&self.message_id)?;
        packet.write(&self.channel)?;

        packet.write_byte_array(&self.data)
    }
}

//
// Serverbound
//

impl_packet_enum!(c2s {
    0x00 => LoginStart,
    0x01 => EncryptionResponse,
    0x02 => LoginPluginResponse,
});

#[derive(Debug, Packet, PacketRead, PacketWrite)]
#[id(0x00)]
pub struct LoginStart {
    pub username: String,
}

#[derive(Debug, Packet, PacketRead, PacketWrite)]
#[id(0x01)]
pub struct EncryptionResponse {
    pub shared_secret: LengthPrefixByteArray,
    pub verify_token: LengthPrefixByteArray,
}

#[derive(Debug, Packet)]
#[id(0x02)]
pub struct LoginPluginResponse {
    pub message_id: v32,
    pub successful: bool,
    pub data: Vec<u8>,
}

impl PacketRead for LoginPluginResponse {
    fn read_data<R: Read>(reader: &mut R, data_len: usize) -> Result<Self> {
        let (message_id, message_id_len) = v32::read(reader)?;
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
        packet.write(&self.message_id)?;
        packet.write(&self.successful)?;

        packet.write_byte_array(&self.data)
    }
}
