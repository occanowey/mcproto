pub mod error;

mod read_ext;
mod write_ext;

pub mod types;
pub use uuid;

pub mod packet;

// TODO: remove and replace with new types.
pub use read_ext::MinecraftReadExt as ReadExt;
pub use write_ext::MinecraftWriteExt as WriteExt;

pub use packet::PacketBuilder;

pub mod net;
