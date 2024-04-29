use super::{impl_packet_enum, Packet, PacketBuilder, PacketRead, PacketWrite};
use crate::{
    error::Result,
    types::{v32, Identifier, LengthPrefixByteArray, McRead},
    ReadExt,
};
use packet_derive::{Packet, PacketRead, PacketWrite};
use std::collections::HashMap;
use std::io::Read;
use uuid::Uuid;

//
// Clientbound
//

impl_packet_enum!(s2c {
    0x00 => ClientboundPluginMessage,
    0x01 => Disconnect,
    0x02 => FinishConfiguration,
    0x03 => ClientboundKeepAlive,
    0x04 => Ping,
    0x05 => RegistryData,
    0x06 => RemoveResourcePack,
    0x07 => AddResourcePack,
    0x08 => FeatureFlags,
    0x09 => UpdateTags,
});

#[derive(Debug, Packet)]
#[id(0x00)]
pub struct ClientboundPluginMessage {
    pub channel: Identifier,
    pub data: Vec<u8>,
}

impl PacketRead for ClientboundPluginMessage {
    fn read_data<R: Read>(reader: &mut R, data_len: usize) -> Result<Self> {
        let (channel, channel_len) = Identifier::read(reader)?;
        let data = reader.read_byte_array(data_len - channel_len)?;

        Ok(ClientboundPluginMessage { channel, data })
    }
}

impl PacketWrite for ClientboundPluginMessage {
    fn write_data(&self, packet: &mut PacketBuilder) -> Result<()> {
        packet.write(&self.channel)?;

        Ok(packet.write_byte_array(&self.data)?)
    }
}

#[derive(Debug, Packet, PacketRead, PacketWrite)]
#[id(0x01)]
pub struct Disconnect {
    // Text Component (NBT)
    pub reason: LengthPrefixByteArray,
}

#[derive(Debug, Packet, PacketRead, PacketWrite)]
#[id(0x02)]
pub struct FinishConfiguration;

#[derive(Debug, Packet, PacketRead, PacketWrite)]
#[id(0x03)]
pub struct ClientboundKeepAlive {
    pub keep_alive_id: i64,
}

#[derive(Debug, Packet, PacketRead, PacketWrite)]
#[id(0x04)]
pub struct Ping {
    pub id: i32,
}

#[derive(Debug, Packet)]
#[id(0x05)]
pub struct RegistryData {
    // NBT - https://wiki.vg/Registry_Data
    // pub registry_data: LengthPrefixByteArray,
    pub registry_data: Vec<u8>,
}

impl PacketRead for RegistryData {
    fn read_data<R: Read>(reader: &mut R, data_len: usize) -> Result<Self> {
        let registry_data = reader.read_byte_array(data_len)?;

        Ok(RegistryData { registry_data })
    }
}

impl PacketWrite for RegistryData {
    fn write_data(&self, packet: &mut PacketBuilder) -> Result<()> {
        Ok(packet.write_byte_array(&self.registry_data)?)
    }
}

#[derive(Debug, Packet)]
#[id(0x06)]
pub struct RemoveResourcePack {
    // None = remove all
    // Some(uuid) = remove specific
    pub uuid: Option<Uuid>,
}

impl PacketRead for RemoveResourcePack {
    fn read_data<R: Read>(reader: &mut R, _data_length: usize) -> Result<Self> {
        let (has_uuid, _) = bool::read(reader)?;

        let uuid = if has_uuid {
            let (uuid, _) = Uuid::read(reader)?;
            Some(uuid)
        } else {
            None
        };

        Ok(RemoveResourcePack { uuid })
    }
}

impl PacketWrite for RemoveResourcePack {
    fn write_data(&self, packet: &mut PacketBuilder) -> Result<()> {
        if let Some(uuid) = &self.uuid {
            packet.write(&true)?;
            packet.write(uuid)?;
        } else {
            packet.write(&false)?;
        }

        Ok(())
    }
}

#[derive(Debug, Packet)]
#[id(0x07)]
pub struct AddResourcePack {
    pub uuid: Uuid,
    pub url: String,
    pub hash: String,
    pub forced: bool,
    // Text Component (NBT)
    pub prompt_message: Option<LengthPrefixByteArray>,
}

impl PacketRead for AddResourcePack {
    fn read_data<R: Read>(reader: &mut R, _data_length: usize) -> Result<Self> {
        let (uuid, _) = Uuid::read(reader)?;
        let (url, _) = String::read(reader)?;
        let (hash, _) = String::read(reader)?;
        let (forced, _) = bool::read(reader)?;

        let (has_prompt_message, _) = bool::read(reader)?;

        let prompt_message = if has_prompt_message {
            let (prompt_message, _) = LengthPrefixByteArray::read(reader)?;
            Some(prompt_message)
        } else {
            None
        };

        Ok(AddResourcePack {
            uuid,
            url,
            hash,
            forced,
            prompt_message,
        })
    }
}

impl PacketWrite for AddResourcePack {
    fn write_data(&self, packet: &mut PacketBuilder) -> Result<()> {
        packet.write(&self.uuid)?;
        packet.write(&self.url)?;
        packet.write(&self.hash)?;
        packet.write(&self.forced)?;

        if let Some(prompt_message) = &self.prompt_message {
            packet.write(&true)?;
            packet.write(prompt_message)?;
        } else {
            packet.write(&false)?;
        }

        Ok(())
    }
}

#[derive(Debug, Packet)]
#[id(0x08)]
pub struct FeatureFlags {
    pub feature_flags: Vec<Identifier>,
}

impl PacketRead for FeatureFlags {
    fn read_data<R: Read>(reader: &mut R, _data_length: usize) -> Result<Self> {
        let mut feature_flags = Vec::new();
        let (feature_flags_count, _) = v32::read(reader)?;
        for _ in 0..feature_flags_count.0 {
            let (feature_flag, _) = Identifier::read(reader)?;
            feature_flags.push(feature_flag);
        }

        Ok(FeatureFlags { feature_flags })
    }
}

impl PacketWrite for FeatureFlags {
    fn write_data(&self, packet: &mut PacketBuilder) -> Result<()> {
        let feature_flags_count = self.feature_flags.len();
        // TODO: return error rather than panic
        assert!(feature_flags_count < (v32::MAX as usize));
        packet.write(&v32(feature_flags_count as _))?;

        for feature_flag in &self.feature_flags {
            packet.write(feature_flag)?;
        }

        Ok(())
    }
}

#[derive(Debug, Packet)]
#[id(0x09)]
pub struct UpdateTags {
    pub tags: HashMap<Identifier, Vec<update_tags::Tag>>,
}

pub mod update_tags {
    use crate::types::{v32, Identifier, McRead, McWrite};

    #[derive(Debug)]
    pub struct Tag {
        pub name: Identifier,
        pub entries: Vec<v32>,
    }

    impl McRead for Tag {
        fn read<R: std::io::Read>(reader: &mut R) -> std::io::Result<(Self, usize)> {
            let (name, name_size) = Identifier::read(reader)?;

            let mut entries = Vec::new();
            let mut entries_size = 0;
            let (entries_count, entries_count_size) = v32::read(reader)?;
            for _ in 0..entries_count.0 {
                let (entry, entry_size) = v32::read(reader)?;

                entries.push(entry);
                entries_size += entry_size;
            }

            let tag = Tag { name, entries };
            Ok((tag, name_size + entries_size + entries_count_size))
        }
    }

    impl McWrite for Tag {
        fn write<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
            self.name.write(writer)?;

            let entries_count = self.entries.len();
            // TODO: return error rather than panic
            assert!(entries_count < (v32::MAX as usize));
            v32(entries_count as _).write(writer)?;

            for entry in &self.entries {
                entry.write(writer)?;
            }

            Ok(())
        }
    }
}

impl PacketRead for UpdateTags {
    fn read_data<R: Read>(reader: &mut R, _data_length: usize) -> Result<Self> {
        let mut tag_map = HashMap::new();
        let (tags_count, _) = v32::read(reader)?;
        for _ in 0..tags_count.0 {
            let (registry, _) = Identifier::read(reader)?;

            let mut tags = Vec::new();
            let (tags_count, _) = v32::read(reader)?;
            for _ in 0..tags_count.0 {
                let (tag, _) = update_tags::Tag::read(reader)?;
                tags.push(tag);
            }

            tag_map.insert(registry, tags);
        }

        Ok(UpdateTags { tags: tag_map })
    }
}

impl PacketWrite for UpdateTags {
    fn write_data(&self, packet: &mut PacketBuilder) -> Result<()> {
        let tag_map_count = self.tags.len();
        // TODO: return error rather than panic
        assert!(tag_map_count < (v32::MAX as usize));
        packet.write(&v32(tag_map_count as _))?;

        for (registry, tags) in &self.tags {
            packet.write(registry)?;

            let tags_count = tags.len();
            // TODO: return error rather than panic
            assert!(tags_count < (v32::MAX as usize));
            packet.write(&v32(tags_count as _))?;

            for tag in tags {
                packet.write(tag)?;
            }
        }

        Ok(())
    }
}

//
// Serverbound
//

impl_packet_enum!(c2s {
    0x00 => ClientInformation,
    0x01 => ServerboundPluginMessage,
    0x02 => AcknowledgeFinishConfiguration,
    0x03 => ServerboundKeepAlive,
    0x04 => Pong,
    0x05 => ResourcePackResponse,
});

#[derive(Debug, Packet, PacketRead, PacketWrite)]
#[id(0x00)]
pub struct ClientInformation {
    pub locale: String,
    pub view_distance: i8,
    pub chat_mode: client_information::ChatMode,
    pub chat_colors: bool,
    pub displayed_skin_parts: client_information::DisplayedSkinParts,
    pub main_hand: client_information::MainHand,
    pub enable_text_filtering: bool,
    pub allow_server_listings: bool,
}

pub mod client_information {
    use crate::types::{v32_enum_read_write, McRead, McWrite};

    #[derive(Debug)]
    pub enum ChatMode {
        Enabled,
        CommandsOnly,
        Hidden,

        Unknown(i32),
    }

    v32_enum_read_write!(
        ChatMode => Unknown
        {
            Enabled = 0,
            CommandsOnly = 1,
            Hidden = 2,
        }
    );

    #[derive(Debug)]
    pub struct DisplayedSkinParts {
        pub cape_enabled: bool,
        pub jacket_enabled: bool,
        pub left_sleeve_enabled: bool,
        pub right_sleeve_enabled: bool,
        pub left_pants_leg_enabled: bool,
        pub right_pants_leg_enabled: bool,
        pub hat_enabled: bool,
    }

    impl McRead for DisplayedSkinParts {
        fn read<R: std::io::Read>(reader: &mut R) -> std::io::Result<(Self, usize)> {
            let (mask, size) = u8::read(reader)?;

            #[rustfmt::skip]
            let displayed_skin_parts = DisplayedSkinParts {
                           cape_enabled: mask & 0b0000001 > 0,
                         jacket_enabled: mask & 0b0000010 > 0,
                    left_sleeve_enabled: mask & 0b0000100 > 0,
                   right_sleeve_enabled: mask & 0b0001000 > 0,
                 left_pants_leg_enabled: mask & 0b0010000 > 0,
                right_pants_leg_enabled: mask & 0b0100000 > 0,
                            hat_enabled: mask & 0b1000000 > 0,
            };

            Ok((displayed_skin_parts, size))
        }
    }

    impl McWrite for DisplayedSkinParts {
        fn write<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
            #[rustfmt::skip]
            #[allow(clippy::identity_op)]
            let mask = (self.           cape_enabled as u8) << 0
                         & (self.         jacket_enabled as u8) << 1
                         & (self.    left_sleeve_enabled as u8) << 2
                         & (self.   right_sleeve_enabled as u8) << 3
                         & (self. left_pants_leg_enabled as u8) << 4
                         & (self.right_pants_leg_enabled as u8) << 5
                         & (self.            hat_enabled as u8) << 6;

            mask.write(writer)
        }
    }

    #[derive(Debug)]
    pub enum MainHand {
        Left,
        Right,
        Unknown(i32),
    }

    v32_enum_read_write!(
        MainHand => Unknown
        {
            Left = 0,
            Right = 1,
        }
    );
}

#[derive(Debug, Packet)]
#[id(0x01)]
pub struct ServerboundPluginMessage {
    pub channel: Identifier,
    pub data: Vec<u8>,
}

impl PacketRead for ServerboundPluginMessage {
    fn read_data<R: Read>(reader: &mut R, data_len: usize) -> Result<Self> {
        let (channel, channel_len) = Identifier::read(reader)?;
        let data = reader.read_byte_array(data_len - channel_len)?;

        Ok(ServerboundPluginMessage { channel, data })
    }
}

impl PacketWrite for ServerboundPluginMessage {
    fn write_data(&self, packet: &mut PacketBuilder) -> Result<()> {
        packet.write(&self.channel)?;

        Ok(packet.write_byte_array(&self.data)?)
    }
}

#[derive(Debug, Packet, PacketRead, PacketWrite)]
#[id(0x02)]
pub struct AcknowledgeFinishConfiguration;

#[derive(Debug, Packet, PacketRead, PacketWrite)]
#[id(0x03)]
pub struct ServerboundKeepAlive {
    pub keep_alive_id: i64,
}

#[derive(Debug, Packet, PacketRead, PacketWrite)]
#[id(0x04)]
pub struct Pong {
    pub id: i32,
}

#[derive(Debug, Packet, PacketRead, PacketWrite)]
#[id(0x05)]
pub struct ResourcePackResponse {
    pub uuid: Uuid,
    pub result: resource_pack_response::Result,
}

mod resource_pack_response {
    use crate::types::v32_enum_read_write;

    #[derive(Debug)]
    pub enum Result {
        SuccessfullyDownloaded,
        Declined,
        FailedToDownload,
        Accepted,
        Downloaded,
        InvalidURL,
        FailedToReload,
        Discarded,

        Unknown(i32),
    }

    v32_enum_read_write!(
        Result => Unknown
        {
            SuccessfullyDownloaded = 0,
            Declined = 1,
            FailedToDownload = 2,
            Accepted = 3,
            Downloaded = 4,
            InvalidURL = 5,
            FailedToReload = 6,
            Discarded = 7,
        }
    );
}
