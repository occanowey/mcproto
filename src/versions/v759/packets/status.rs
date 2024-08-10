//
// Clientbound
//

pub mod s2c {
    use crate::packet::prelude::*;

    impl_packets_enum![StatusResponse, PingResponse];

    #[derive(Debug, Packet, BufType)]
    #[packet(id = 0x00)]
    pub struct StatusResponse {
        pub response: String,
    }

    #[derive(Debug, Packet, BufType)]
    #[packet(id = 0x01)]
    pub struct PingResponse {
        pub payload: i64,
    }
}

//
// Serverbound
//

pub mod c2s {
    use crate::packet::prelude::*;

    impl_packets_enum![StatusRequest, PingRequest];

    #[derive(Debug, Packet, BufType)]
    #[packet(id = 0x00)]
    pub struct StatusRequest;

    #[derive(Debug, Packet, BufType)]
    #[packet(id = 0x01)]
    pub struct PingRequest {
        pub payload: i64,
    }
}
