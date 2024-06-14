use crate::message::Message;
use anyhow::Result;
use std::{
    marker::PhantomData,
    net::{Ipv4Addr, SocketAddr, UdpSocket},
};

pub struct DnsClient;
pub struct DnsService;

pub struct DnsSocket<T> {
    socket: UdpSocket,
    mode: PhantomData<T>,
}

impl<T> DnsSocket<T> {}

impl DnsSocket<DnsService> {
    pub fn listen(addr: &str) -> Result<Self> {
        let socket = UdpSocket::bind(addr)?;
        Ok(Self {
            socket,
            mode: PhantomData,
        })
    }

    pub fn read(&self) -> Result<(Message, SocketAddr)> {
        let mut buf = [0; 512];
        let (size, addr) = self.socket.recv_from(&mut buf)?;
        anyhow::ensure!(size > 12, "Packet is not long enough: {size}");

        let msg = Message::try_from(buf.as_ref())?;
        anyhow::ensure!(msg.is_query());

        Ok((msg, addr))
    }

    pub fn send_to(&self, m: &Message, addr: SocketAddr) -> Result<()> {
        let buf = m.flush();
        let sent = self.socket.send_to(&buf, addr)?;
        anyhow::ensure!(sent == buf.len());
        Ok(())
    }
}

impl DnsSocket<DnsClient> {
    pub fn connect(addr: &str) -> Result<Self> {
        let socket = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 0))?;
        socket.connect(addr)?;
        Ok(Self {
            socket,
            mode: PhantomData,
        })
    }

    pub fn recv(&self) -> Result<Message> {
        let mut buf = [0; 512];
        self.socket.recv(&mut buf)?;
        let msg = Message::try_from(buf.as_ref())?;
        Ok(msg)
    }

    pub fn send(&self, m: &Message) -> Result<()> {
        let buf = m.flush();
        let sent = self.socket.send(&buf)?;
        anyhow::ensure!(sent == buf.len());
        Ok(())
    }
}
