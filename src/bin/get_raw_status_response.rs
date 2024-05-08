use std::{
    env,
    error::Error,
    net::{TcpStream, ToSocketAddrs},
    time::Instant,
};

use mcproto::{
    net::handler_from_stream,
    packet::{
        handshaking::{Handshake, NextState},
        status::{PingRequest, PingResponse, StatusRequest, StatusResponse},
    },
};

fn main() -> Result<(), Box<dyn Error>> {
    let address = env::args()
        .nth(1)
        .expect("address required")
        .to_socket_addrs()
        .expect("address is invalid")
        .next()
        .unwrap();

    let server = TcpStream::connect(address)?;
    let mut handler = handler_from_stream(server)?;

    handler.write(Handshake {
        protocol_version: 110,
        server_address: "localhost".into(),
        server_port: 25565,
        next_state: NextState::Status,
        forge: None,
    })?;

    let mut handler = handler.status();

    handler.write(StatusRequest)?;
    let response: StatusResponse = handler.read()?;

    let now = Instant::now();

    handler.write(PingRequest { payload: 1 })?;
    let pong: PingResponse = handler.read()?;

    let duration = now.elapsed().as_millis();

    if pong.payload != 1 {
        println!("! server replied with different payload")
    }

    println!(
        "---------- status response (took {}ms) ----------",
        duration
    );
    println!("{}", response.response);
    println!("-------------------------------------------------");

    Ok(())
}
