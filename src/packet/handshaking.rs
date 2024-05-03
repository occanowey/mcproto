use super::{impl_packet_enum, Packet, PacketBuilder, PacketRead, PacketWrite};
use crate::{
    error::Result,
    types::{proxy::i32_as_v32, v32_enum_read_write, McRead},
};
use packet_derive::Packet;
use std::io::Read;

//
// Serverbound
//

impl_packet_enum!(c2s {
    0x00 => Handshake
});

// i hate it here
// https://wiki.vg/Minecraft_Forge_Handshake
#[derive(Debug)]
pub enum ForgeHandshake {
    // forge 1.7 - 1.12
    Version1,

    // forge 1.13+
    Version2,
}

impl ForgeHandshake {
    fn separate_address(address: String) -> (String, Option<Self>) {
        if !address.contains('\0') {
            (address, None)
        } else {
            let (address, fml) = address.split_once('\0').unwrap();

            let forge = match fml {
                "FML\0" => Some(Self::Version1),
                "FML2\0" => Some(Self::Version2),

                // should definately warn about this somehow
                _ => None,
            };

            (address.to_owned(), forge)
        }
    }

    fn net_id(&self) -> &str {
        match self {
            Self::Version1 => "\0FML\0",
            Self::Version2 => "\0FML2\0",
        }
    }
}

#[derive(Debug)]
pub enum NextState {
    Status,
    Login,

    Unknown(i32),
}

v32_enum_read_write!(
    NextState => Unknown
    {
        Status = 1,
        Login = 2,
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
        format!(
            "{}{}",
            self.server_address,
            self.forge.as_ref().map_or("", |f| f.net_id()),
        )
    }
}

impl PacketRead for Handshake {
    fn read_data<R: Read>(reader: &mut R, _: usize) -> Result<Handshake> {
        // todo: maybe handle legacy ping?
        let protocol_version = i32_as_v32::read(reader, 0)?.0;
        let server_address = String::read(reader)?.0;
        let server_port = u16::read(reader)?.0;
        let next_state = NextState::read(reader)?.0;

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
    fn write_data(&self, packet: &mut PacketBuilder) -> Result<()> {
        i32_as_v32::write(packet, &self.protocol_version)?;
        packet.write(&self.modified_address())?;
        packet.write(&self.server_port)?;
        Ok(packet.write(&self.next_state)?)
    }
}
