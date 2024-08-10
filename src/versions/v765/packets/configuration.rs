use super::super::super::v764::packets::configuration as prev;

//
// Clientbound
//

pub mod s2c {
    use crate::packet::prelude::*;

    impl_packets_enum![
        ClientboundPluginMessage,
        Disconnect,
        FinishConfiguration,
        ClientboundKeepAlive,
        Ping,
        RegistryData,
        RemoveResourcePack,
        AddResourcePack,
        FeatureFlags,
        UpdateTags,
    ];

    // 0x00
    pub use super::prev::s2c::ClientboundPluginMessage;

    // 0x01
    pub use super::prev::s2c::Disconnect;

    // 0x02
    pub use super::prev::s2c::FinishConfiguration;

    // 0x03
    pub use super::prev::s2c::ClientboundKeepAlive;

    // 0x04
    pub use super::prev::s2c::Ping;

    // 0x05
    pub use super::prev::s2c::RegistryData;

    #[derive(Debug, Packet, BufPacket)]
    #[packet(id = 0x06)]
    pub struct RemoveResourcePack {
        // None = remove all
        // Some(uuid) = remove specific
        pub uuid: Option<Uuid>,
    }

    #[derive(Debug, Packet, BufPacket)]
    #[packet(id = 0x07)]
    pub struct AddResourcePack {
        pub uuid: Uuid,
        pub url: String,
        pub hash: String,
        pub forced: bool,
        // Text Component (NBT)
        #[packet(with = "option_length_prefix_bytes")]
        pub prompt_message: Option<Vec<u8>>,
    }

    #[derive(Debug, Packet, BufPacket)]
    #[packet(id = 0x08)]
    pub struct FeatureFlags {
        #[packet(with = "length_prefix_array")]
        pub feature_flags: Vec<Identifier>,
    }

    #[derive(Debug, Packet)]
    #[packet(id = 0x09)]
    pub struct UpdateTags {
        pub tags: HashMap<Identifier, Vec<update_tags::Tag>>,
    }

    pub mod update_tags {
        use crate::types::ReadError;
        use crate::types::{proxy::length_prefix_array, v32, BufType, Identifier};

        #[derive(Debug)]
        pub struct Tag {
            pub name: Identifier,
            pub entries: Vec<i32>,
        }

        impl BufType for Tag {
            fn buf_read_len<B: bytes::Buf>(buf: &mut B) -> Result<(Self, usize), ReadError> {
                let (name, name_size) = Identifier::buf_read_len(buf)?;
                let (entries, entries_size): (Vec<v32>, _) =
                    length_prefix_array::buf_read_len(buf)?;

                let tag = Tag {
                    name,
                    entries: entries.into_iter().map(|i| i.0).collect(), // :/
                };
                Ok((tag, name_size + entries_size))
            }

            fn buf_write<B: bytes::BufMut>(&self, buf: &mut B) {
                self.name.buf_write(buf);
                let entries = self.entries.iter().map(|i| v32(*i)).collect::<Vec<_>>(); // still :/
                length_prefix_array::buf_write(&entries, buf)
            }
        }
    }

    impl PacketRead for UpdateTags {
        fn read_body<B: Buf>(data: &mut B) -> Result<Self, ReadError> {
            let mut tag_map = HashMap::new();
            let tags_count = i32_as_v32::buf_read(data)?;
            for _ in 0..tags_count {
                let registry = Identifier::buf_read(data)?;
                let tags = length_prefix_array::buf_read(data)?;

                tag_map.insert(registry, tags);
            }

            Ok(UpdateTags { tags: tag_map })
        }
    }

    impl PacketWrite for UpdateTags {
        fn write_body<B: BufMut>(&self, buf: &mut B) {
            let tag_map_count = self.tags.len();
            // TODO: return error rather than panic
            assert!(tag_map_count < (crate::types::v32::MAX as usize));
            i32_as_v32::buf_write(&(tag_map_count as _), buf);

            for (registry, tags) in &self.tags {
                registry.buf_write(buf);
                length_prefix_array::buf_write(tags, buf);
            }
        }
    }
}

//
// Serverbound
//

pub mod c2s {
    use crate::packet::prelude::*;

    impl_packets_enum![
        ClientInformation,
        ServerboundPluginMessage,
        FinishConfiguration,
        ServerboundKeepAlive,
        Pong,
        ResourcePackResponse,
    ];

    // 0x00
    pub use super::prev::c2s::{client_information, ClientInformation};

    // 0x01
    pub use super::prev::c2s::ServerboundPluginMessage;

    // 0x02
    pub use super::prev::c2s::FinishConfiguration;

    // 0x03
    pub use super::prev::c2s::ServerboundKeepAlive;

    // 0x04
    pub use super::prev::c2s::Pong;

    #[derive(Debug, Packet, BufPacket)]
    #[packet(id = 0x06)]
    pub struct ResourcePackResponse {
        pub uuid: Uuid,
        pub result: resource_pack_response::Result,
    }

    mod resource_pack_response {
        use crate::types::v32_prefix_enum;

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

        v32_prefix_enum!(
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
}
