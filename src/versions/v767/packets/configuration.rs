use super::super::super::v766::packets::configuration as prev;

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
        CustomReportDetails,
        ServerLinks,
    ];

    // 0x00
    pub use super::prev::s2c::CookieRequest;

    // 0x01
    pub use super::prev::s2c::ClientboundPluginMessage;

    // 0x02
    pub use super::prev::s2c::Disconnect;

    // 0x03
    pub use super::prev::s2c::FinishConfiguration;

    // 0x04
    pub use super::prev::s2c::ClientboundKeepAlive;

    // 0x05
    pub use super::prev::s2c::Ping;

    // 0x06
    pub use super::prev::s2c::ResetChat;

    // 0x07
    pub use super::prev::s2c::{registry_data, RegistryData};

    // 0x08
    pub use super::prev::s2c::RemoveResourcePack;

    // 0x09
    pub use super::prev::s2c::AddResourcePack;

    // 0x0a
    pub use super::prev::s2c::StoreCookie;

    // 0x0b
    pub use super::prev::s2c::Transfer;

    // 0x0c
    pub use super::prev::s2c::FeatureFlags;

    // 0x0d
    pub use super::prev::s2c::{update_tags, UpdateTags};

    // 0x0e
    pub use super::prev::s2c::{known_packs, ClientboundKnownPacks};

    #[derive(Debug, Packet, BufType)]
    #[packet(id = 0x0f)]
    pub struct CustomReportDetails {
        #[buftype(with = "length_prefix_array")]
        pub details: Vec<custom_report_details::Detail>,
    }

    pub mod custom_report_details {
        use crate::packet::prelude::*;

        #[derive(Debug, BufType)]
        pub struct Detail {
            pub title: String,
            pub description: String,
        }
    }

    #[derive(Debug, Packet, BufType)]
    #[packet(id = 0x10)]
    pub struct ServerLinks {
        #[buftype(with = "length_prefix_array")]
        pub links: Vec<server_links::ServerLink>,
    }

    pub mod server_links {
        use crate::packet::prelude::*;

        #[derive(Debug)]
        pub enum ServerLinkLabel {
            BugReport,
            CommunityGuidelines,
            Support,
            Status,
            Feedback,
            Community,
            Website,
            Forums,
            News,
            Announcements,
            Unknown(i32),
            Custom(String),
        }

        #[derive(Debug, BufType)]
        pub struct ServerLink {
            pub label: ServerLinkLabel,
            pub url: String,
        }

        impl BufType for ServerLinkLabel {
            fn buf_read_len<B: bytes::Buf>(buf: &mut B) -> Result<(Self, usize), ReadError> {
                let (is_built_in, is_built_in_len) = bool::buf_read_len(buf)?;
                let (label, label_len) = if is_built_in {
                    let (label, label_len) = i32_as_v32::buf_read_len(buf)?;

                    let label = match label {
                        0 => ServerLinkLabel::BugReport,
                        1 => ServerLinkLabel::CommunityGuidelines,
                        2 => ServerLinkLabel::Support,
                        3 => ServerLinkLabel::Status,
                        4 => ServerLinkLabel::Feedback,
                        5 => ServerLinkLabel::Community,
                        6 => ServerLinkLabel::Website,
                        7 => ServerLinkLabel::Forums,
                        8 => ServerLinkLabel::News,
                        9 => ServerLinkLabel::Announcements,
                        other => ServerLinkLabel::Unknown(other),
                    };

                    (label, label_len)
                } else {
                    let (label, label_len) = String::buf_read_len(buf)?;
                    (ServerLinkLabel::Custom(label), label_len)
                };

                Ok((label, is_built_in_len + label_len))
            }

            fn buf_write<B: bytes::BufMut>(&self, buf: &mut B) {
                match &self {
                    ServerLinkLabel::Custom(label) => {
                        bool::buf_write(&false, buf);
                        label.buf_write(buf);
                    }

                    other => {
                        bool::buf_write(&true, buf);

                        let id = match other {
                            ServerLinkLabel::BugReport => 0,
                            ServerLinkLabel::CommunityGuidelines => 1,
                            ServerLinkLabel::Support => 2,
                            ServerLinkLabel::Status => 3,
                            ServerLinkLabel::Feedback => 4,
                            ServerLinkLabel::Community => 5,
                            ServerLinkLabel::Website => 6,
                            ServerLinkLabel::Forums => 7,
                            ServerLinkLabel::News => 8,
                            ServerLinkLabel::Announcements => 9,
                            ServerLinkLabel::Unknown(other) => *other,
                            ServerLinkLabel::Custom(_) => unreachable!(),
                        };
                        i32_as_v32::buf_write(&id, buf);
                    }
                }
            }
        }
    }
}

//
// Serverbound
//

pub use prev::c2s;
