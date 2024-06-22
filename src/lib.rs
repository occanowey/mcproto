pub use uuid;

pub mod side {
    pub trait NetworkSide {}

    pub struct Server;
    impl NetworkSide for Server {}

    pub struct Client;
    impl NetworkSide for Client {}
}

pub mod error;

pub mod handler;
pub mod handshake;
pub mod packet;
pub mod state;
pub mod types;
pub mod versions;
