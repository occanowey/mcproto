use std::{env, error::Error, net::ToSocketAddrs, time::Instant};

use mcproto::{
    handshake::{Handshake, NextState},
    role,
    stdio::{self, StdIoConnection},
    versions::latest::{
        packets::status::{
            c2s::{PingRequest, StatusRequest},
            s2c::{PingResponse, StatusResponse},
        },
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

    let mut connection: StdIoConnection<role::Client, _> = stdio::connect_stdio_stream(address)?;

    connection.write_packet(Handshake {
        protocol_version: 110,
        server_address: "localhost".into(),
        server_port: 25565,
        next_state: NextState::Status,
        forge: None,
    })?;

    let mut connection = connection.next_state::<states::StatusState>();

    connection.write_packet(StatusRequest)?;
    let response: StatusResponse = connection.expect_next_packet()?;

    let now = Instant::now();

    connection.write_packet(PingRequest { payload: 1 })?;
    let pong: PingResponse = connection.expect_next_packet()?;

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
