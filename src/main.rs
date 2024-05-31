#![warn(clippy::pedantic)]

mod message;
use anyhow::Result;
use std::net::UdpSocket;

fn main() -> Result<()> {
    println!("Logs from your program will appear here!");

    let udp_socket = UdpSocket::bind("127.0.0.1:2053").expect("Failed to bind to address");
    let mut buf = [0; 512];

    loop {
        let (size, source) = udp_socket.recv_from(&mut buf)?;
        anyhow::ensure!(size > 12, "Packet is not long enough: {size}");

        dbg!(size);

        let response = [];
        udp_socket.send_to(&response, source)?;
    }
}
