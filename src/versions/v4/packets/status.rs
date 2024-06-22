//
// Clientbound
//

pub mod s2c {
    use crate::packet::prelude::*;

    impl_packets_enum![Response, Pong];

    #[derive(Debug, Packet, PacketRead, PacketWrite)]
    #[packet(id = 0x00)]
    pub struct Response {
        pub response: String,
    }

    #[derive(Debug, Packet, PacketRead, PacketWrite)]
    #[packet(id = 0x01)]
    pub struct Pong {
        pub payload: i64,
    }
}

//
// Serverbound
//

pub mod c2s {
    use crate::packet::prelude::*;

    impl_packets_enum![Request, Ping];

    #[derive(Debug, Packet, PacketRead, PacketWrite)]
    #[packet(id = 0x00)]
    pub struct Request;

    #[derive(Debug, Packet, PacketRead, PacketWrite)]
    #[packet(id = 0x01)]
    pub struct Ping {
        pub payload: i64,
    }
}
