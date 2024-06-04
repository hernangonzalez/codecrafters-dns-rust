mod message;
mod parser;
mod writer;
use anyhow::Result;
use message::{Message, Question};
use std::net::UdpSocket;

fn main() -> Result<()> {
    println!("Logs from your program will appear here!");

    let udp_socket = UdpSocket::bind("127.0.0.1:2053").expect("Failed to bind to address");
    let mut buf = [0; 512];

    loop {
        let (size, source) = udp_socket.recv_from(&mut buf)?;
        // let buf = buf.clone();
        anyhow::ensure!(size > 12, "Packet is not long enough: {size}");

        let msg = Message::try_from(buf.as_ref())?;
        anyhow::ensure!(msg.is_query());

        let res = look_up(msg)?;
        let buf = res.flush();
        udp_socket.send_to(&buf, source)?;
    }
}

fn look_up(query: Message) -> Result<Message> {
    let mut msg = Message::new_response(query);
    let q = Question::new_aa("codecrafters.io");
    msg.set_questions(vec![q]);
    Ok(msg)
}
