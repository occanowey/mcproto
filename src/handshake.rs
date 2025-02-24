use crate::packet::prelude::*;
use crate::role;
use crate::state::{self, impl_state};

impl_packets_enum![Handshake];

// i hate it here
// https://wiki.vg/Minecraft_Forge_Handshake
#[derive(Debug)]
pub enum ForgeHandshake {
    // forge 1.7 - 1.12
    Version1,

    // forge 1.13 - ??
    Version2,

    // forge ??
    Version3,

    Other(String),
}

impl ForgeHandshake {
    fn separate_address(full_address: String) -> (String, Option<Self>) {
        let (address, version) = full_address.split_once('\0').unzip();

        let version = version.map(|v| match v {
            "FML\0" => Self::Version1,
            "FML2\0" => Self::Version2,
            "FML3\0" => Self::Version3,

            other => Self::Other(other.to_owned()),
        });

        let address = address.map(ToOwned::to_owned).unwrap_or(full_address);

        (address, version)
    }

    fn net_id(&self) -> &str {
        match self {
            Self::Version1 => "FML\0",
            Self::Version2 => "FML2\0",
            Self::Version3 => "FML3\0",

            Self::Other(other) => other.as_str(),
        }
    }
}

#[derive(Debug)]
pub enum NextState {
    Status,
    Login,
    Transfer,

    Unknown(i32),
}

v32_prefix_enum!(
    NextState => Unknown
    {
        Status = 1,
        Login = 2,
        Transfer = 3,
    }
);

#[derive(Debug, Packet)]
#[packet(id = 0x00)]
pub struct Handshake {
    pub protocol_version: i32,
    pub server_address: String,
    pub server_port: u16,
    pub next_state: NextState,

    pub forge: Option<ForgeHandshake>,
}

impl Handshake {
    fn modified_address(&self) -> String {
        let mut address = self.server_address.clone();
        if let Some(forge) = &self.forge {
            address.push('\0');
            address.push_str(forge.net_id());
        }

        address
    }
}

impl PacketRead for Handshake {
    fn read_body<B: Buf>(data: &mut B) -> Result<Handshake, ReadError> {
        // todo: maybe handle legacy ping?
        let protocol_version = i32_as_v32::buf_read(data)?;
        let server_address = String::buf_read(data)?;
        let server_port = u16::buf_read(data)?;
        let next_state = NextState::buf_read(data)?;

        let (server_address, forge) = ForgeHandshake::separate_address(server_address);

        Ok(Handshake {
            protocol_version,
            server_address,
            server_port,
            next_state,
            forge,
        })
    }
}

impl PacketWrite for Handshake {
    fn write_body<B: BufMut>(&self, buf: &mut B) {
        i32_as_v32::buf_write(&self.protocol_version, buf);
        self.modified_address().buf_write(buf);
        self.server_port.buf_write(buf);
        self.next_state.buf_write(buf);
    }
}

impl_state!(HandshakingState("handshaking"), [], c2s[Handshake]);

impl state::RoleStatePackets<role::Server> for HandshakingState {
    type RecvPacket = Packets;
}
