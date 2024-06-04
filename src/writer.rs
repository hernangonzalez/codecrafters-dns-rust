use crate::{
    message::{
        header::{Authoritative, QueryMode, Recursion, Truncation},
        Answer, Data, Header, Question,
    },
    Message,
};
use bytes::{BufMut, BytesMut};

trait Writer {
    fn write(&self, buf: &mut BytesMut);
}

impl Writer for &str {
    fn write(&self, buf: &mut BytesMut) {
        self.split('.').for_each(|l| {
            buf.put_u8(l.len() as u8);
            buf.put(l.as_bytes())
        });
        buf.put_u8(0);
    }
}

impl<T: Writer> Writer for Vec<T> {
    fn write(&self, buf: &mut BytesMut) {
        self.iter().for_each(|i| i.write(buf))
    }
}

impl Writer for Header {
    fn write(&self, buf: &mut BytesMut) {
        buf.put_u16(self.id.0);

        let mut flags = 0u8;
        if self.qr == QueryMode::Response {
            flags |= 0b1_0000000;
        }
        flags |= 0b01111000 & (self.op_code.0 << 3);
        if self.aa == Authoritative::Owned {
            flags |= 0b0000_0100;
        }
        if self.tc == Truncation::Truncated {
            flags |= 0b0000_0010;
        }
        if self.rd == Recursion::Enabled {
            flags |= 0b0000_0001;
        }
        buf.put_u8(flags);

        flags = 0u8;
        if self.ra == Recursion::Enabled {
            flags &= 0b1_0000000;
        }
        flags |= 0b0000_1111 & self.r_code.0;
        buf.put_u8(flags);

        buf.put_u16(self.qd_count);
        buf.put_u16(self.an_count);
        buf.put_u16(self.ns_count);
        buf.put_u16(self.ar_count);
    }
}

impl Writer for Question {
    fn write(&self, buf: &mut BytesMut) {
        self.name.as_str().write(buf);
        buf.put_u16(self.record as u16);
        buf.put_u16(self.class as u16);
    }
}

impl Writer for Data {
    fn write(&self, buf: &mut BytesMut) {
        match self {
            Self::Ipv4(ip) => buf.put_slice(&ip.octets()),
        }
    }
}

impl Writer for Answer {
    fn write(&self, buf: &mut BytesMut) {
        self.name().write(buf);
        buf.put_u16(self.record() as u16);
        buf.put_u16(self.class() as u16);
        buf.put_u32(self.ttl());
        buf.put_u16(self.len());
        self.data().write(buf);
    }
}

impl Message {
    pub fn flush(&self) -> BytesMut {
        let mut buf = BytesMut::with_capacity(12);
        self.header().write(&mut buf);
        self.questions().write(&mut buf);
        self.answers().write(&mut buf);
        buf
    }
}
