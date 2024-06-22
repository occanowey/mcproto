use std::{
    env,
    error::Error,
    net::{TcpStream, ToSocketAddrs},
    time::Instant,
};

use mcproto::{
    connection::connection_from_stream,
    handshake::{Handshake, NextState},
    versions::latest::{
        packets::status::{PingRequest, PingResponse, StatusRequest, StatusResponse},
        states,
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
    let mut connection = connection_from_stream(server)?;

    connection.write(Handshake {
        protocol_version: 110,
        server_address: "localhost".into(),
        server_port: 25565,
        next_state: NextState::Status,
        forge: None,
    })?;

    let mut connection = connection.next_state::<states::StatusState>();

    connection.write(StatusRequest)?;
    let response: StatusResponse = connection.read()?;

    let now = Instant::now();

    connection.write(PingRequest { payload: 1 })?;
    let pong: PingResponse = connection.read()?;

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
