//
// Clientbound
//
pub mod s2c {
    use crate::packet::prelude::*;

    impl_packets_enum![Disconnect];

    #[derive(Debug, Packet, PacketRead, PacketWrite)]
    #[packet(id = 0x1A)]
    pub struct Disconnect {
        pub reason: String,
    }
}

//
// Serverbound
//
#[allow(unused_mut)]
pub mod c2s {
    use crate::packet::prelude::*;

    impl_packets_enum![];
}
