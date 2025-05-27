pub use packet_derive;
pub use uuid;

pub mod role {
    pub trait ConnectionRole {}

    pub struct Server;
    impl ConnectionRole for Server {}

    pub struct Client;
    impl ConnectionRole for Client {}
}

pub mod error;

pub mod connection;
pub mod handshake;
pub mod packet;
pub mod state;
pub mod types;
pub mod versions;

pub mod stdio;
#[cfg(feature = "tokio")]
pub mod tokio;
