pub mod side {
    pub trait NetworkSide {}

    pub struct Server;
    impl NetworkSide for Server {}

    pub struct Client;
    impl NetworkSide for Client {}
}

mod encryption;
mod handler;

pub use handler::{handler_from_stream, NetworkHandler};
