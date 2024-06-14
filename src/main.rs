mod message;
mod parser;
mod socket;
mod writer;
use anyhow::{Context, Result};
use message::{data::Data, route::Route, Message};
use socket::DnsSocket;
use std::{env, net::Ipv4Addr};

#[derive(Debug)]
struct Args {
    resolver: Option<String>,
}

fn parse_args() -> Result<Args> {
    let mut resolver = None;
    let mut all = env::args().skip(1);
    while let Some(arg) = all.next() {
        match arg.as_str() {
            "--resolver" => {
                let ip = all.next().context("Missing resolver IPv4")?;
                resolver = Some(ip)
            }
            u => println!("Unknown argument: {u}"),
        }
    }
    Ok(Args { resolver })
}

fn main() -> Result<()> {
    println!("Starting DNS...");

    let args = parse_args()?;
    let srv = DnsSocket::listen("127.0.0.1:2053")?;

    while let Ok((q, addr)) = srv.read() {
        let res: Message = if let Some(ref addr) = args.resolver {
            resolve_from(addr, &q)?
        } else {
            look_up_local(q)?
        };

        srv.send_to(&res, addr)?;
    }

    Ok(())
}

fn resolve_from(addr: &str, msg: &Message) -> Result<Message> {
    let client = DnsSocket::connect(addr)?;

    let mut answers = vec![];
    for q in msg.questions().iter() {
        let mut query = Message::new(*msg.header());
        let qs = vec![q.clone()];
        query.set_questions(qs)?;

        client.send(&query)?;

        let msg = client.recv()?;

        let mut ans: Vec<Route> = msg.answers().clone();
        answers.append(&mut ans)
    }

    let mut msg = Message::new_response(msg);
    msg.set_answers(answers)?;
    Ok(msg)
}

fn look_up_local(query: Message) -> Result<Message> {
    let mut msg = Message::new_response(&query);

    let ip = Ipv4Addr::new(8, 8, 8, 8);
    let data = Data::Ipv4(ip);
    let ans: Vec<_> = query
        .questions()
        .iter()
        .cloned()
        .map(|d| Route::new(d, 60, data))
        .collect();
    msg.set_answers(ans)?;

    Ok(msg)
}
