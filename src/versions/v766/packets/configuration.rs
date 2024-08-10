use super::super::super::v765::packets::configuration as prev;

//
// Clientbound
//

pub mod s2c {
    use crate::packet::prelude::*;

    impl_packets_enum![
        CookieRequest,
        ClientboundPluginMessage,
        Disconnect,
        FinishConfiguration,
        ClientboundKeepAlive,
        Ping,
        ResetChat,
        RegistryData,
        RemoveResourcePack,
        AddResourcePack,
        StoreCookie,
        Transfer,
        FeatureFlags,
        UpdateTags,
        ClientboundKnownPacks,
    ];

    #[derive(Debug, Packet, BufType)]
    #[packet(id = 0x00)]
    pub struct CookieRequest {
        pub key: Identifier,
    }

    #[derive(Debug, Packet, BufType)]
    #[packet(id = 0x01)]
    pub struct ClientboundPluginMessage {
        pub channel: Identifier,
        #[buftype(with = "remaining_bytes")]
        pub data: Vec<u8>,
    }

    #[derive(Debug, Packet, BufType)]
    #[packet(id = 0x02)]
    pub struct Disconnect {
        // Text Component (NBT)
        #[buftype(with = "length_prefix_bytes")]
        pub reason: Vec<u8>,
    }

    #[derive(Debug, Packet, BufType)]
    #[packet(id = 0x03)]
    pub struct FinishConfiguration;

    #[derive(Debug, Packet, BufType)]
    #[packet(id = 0x04)]
    pub struct ClientboundKeepAlive {
        pub keep_alive_id: i64,
    }

    #[derive(Debug, Packet, BufType)]
    #[packet(id = 0x05)]
    pub struct Ping {
        pub id: i32,
    }

    #[derive(Debug, Packet, BufType)]
    #[packet(id = 0x06)]
    pub struct ResetChat;

    #[derive(Debug, Packet, BufType)]
    #[packet(id = 0x07)]
    pub struct RegistryData {
        pub registry_id: Identifier,
        // #[buftype(with = "length_prefix_array")]
        // pub entries: Vec<registry_data::Entry>,

        // TODO: properly decode entry data
        #[buftype(with = "remaining_bytes")]
        pub _entries_data: Vec<u8>,
    }

    pub mod registry_data {
        use crate::types::{BufType, Identifier, ReadError};

        #[derive(Debug)]
        pub struct Entry {
            pub entry_id: Identifier,

            // NBT - https://wiki.vg/Registry_Data
            // length isn't encoded, need to infer from compound close tag?
            // TODO: properly decode entry data
            pub data: Option<Vec<u8>>,
        }

        impl BufType for Entry {
            fn buf_read_len<B: bytes::Buf>(_buf: &mut B) -> Result<(Self, usize), ReadError> {
                todo!()
            }

            fn buf_write<B: bytes::BufMut>(&self, _buf: &mut B) {
                todo!()
            }
        }
    }

    #[derive(Debug, Packet, BufType)]
    #[packet(id = 0x08)]
    pub struct RemoveResourcePack {
        // None = remove all
        // Some(uuid) = remove specific
        pub uuid: Option<Uuid>,
    }

    #[derive(Debug, Packet, BufType)]
    #[packet(id = 0x09)]
    pub struct AddResourcePack {
        pub uuid: Uuid,
        pub url: String,
        pub hash: String,
        pub forced: bool,
        // Text Component (NBT)
        #[buftype(with = "option_length_prefix_bytes")]
        pub prompt_message: Option<Vec<u8>>,
    }

    #[derive(Debug, Packet, BufType)]
    #[packet(id = 0x0a)]
    pub struct StoreCookie {
        pub key: Identifier,
        // TODO: check if length is encoded
        #[buftype(with = "remaining_bytes")]
        pub payload: Vec<u8>,
    }

    #[derive(Debug, Packet, BufType)]
    #[packet(id = 0x0b)]
    pub struct Transfer {
        host: String,
        #[buftype(with = "i32_as_v32")]
        port: i32,
    }

    #[derive(Debug, Packet, BufType)]
    #[packet(id = 0x0c)]
    pub struct FeatureFlags {
        #[buftype(with = "length_prefix_array")]
        pub feature_flags: Vec<Identifier>,
    }

    #[derive(Debug, Packet)]
    #[packet(id = 0x0d)]
    pub struct UpdateTags {
        pub tags: HashMap<Identifier, Vec<update_tags::Tag>>,
    }

    pub use super::prev::s2c::update_tags;

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

    #[derive(Debug, Packet, BufType)]
    #[packet(id = 0x0e)]
    pub struct ClientboundKnownPacks {
        #[buftype(with = "length_prefix_array")]
        pub known_packs: Vec<known_packs::KnownPack>,
    }

    pub mod known_packs {
        use crate::types::{BufType, ReadError};

        #[derive(Debug)]
        pub struct KnownPack {
            pub namespace: String,
            pub id: String,
            pub version: String,
        }

        impl BufType for KnownPack {
            fn buf_read_len<B: bytes::Buf>(buf: &mut B) -> Result<(Self, usize), ReadError> {
                let (namespace, namespace_len) = String::buf_read_len(buf)?;
                let (id, id_len) = String::buf_read_len(buf)?;
                let (version, version_len) = String::buf_read_len(buf)?;

                let known_pack = KnownPack {
                    namespace,
                    id,
                    version,
                };

                Ok((known_pack, namespace_len + id_len + version_len))
            }

            fn buf_write<B: bytes::BufMut>(&self, buf: &mut B) {
                self.namespace.buf_write(buf);
                self.id.buf_write(buf);
                self.version.buf_write(buf);
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
        CookieResponse,
        ServerboundPluginMessage,
        AcknowledgeFinishConfiguration,
        ServerboundKeepAlive,
        Pong,
        ResourcePackResponse,
        ServerboundKnownPacks,
    ];

    // 0x00
    pub use super::prev::c2s::{client_information, ClientInformation};

    #[derive(Debug, Packet, BufType)]
    #[packet(id = 0x01)]
    pub struct CookieResponse {
        pub key: Identifier,
        #[buftype(with = "option_length_prefix_bytes")]
        pub payload: Option<Vec<u8>>,
    }

    #[derive(Debug, Packet, BufType)]
    #[packet(id = 0x02)]
    pub struct ServerboundPluginMessage {
        pub channel: Identifier,
        #[buftype(with = "remaining_bytes")]
        pub data: Vec<u8>,
    }

    #[derive(Debug, Packet, BufType)]
    #[packet(id = 0x03)]
    pub struct AcknowledgeFinishConfiguration;

    #[derive(Debug, Packet, BufType)]
    #[packet(id = 0x04)]
    pub struct ServerboundKeepAlive {
        pub keep_alive_id: i64,
    }

    #[derive(Debug, Packet, BufType)]
    #[packet(id = 0x05)]
    pub struct Pong {
        pub id: i32,
    }

    #[derive(Debug, Packet, BufType)]
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

    #[derive(Debug, Packet, BufType)]
    #[packet(id = 0x07)]
    pub struct ServerboundKnownPacks {
        #[buftype(with = "length_prefix_array")]
        pub known_packs: Vec<known_packs::KnownPack>,
    }

    pub use super::s2c::known_packs;
}
