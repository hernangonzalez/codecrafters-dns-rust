use crate::{
    message::{
        data::Data,
        domain::Domain,
        header::{Authoritative, QueryMode, Recursion, Truncation},
        route::Route,
        Header,
    },
    Message,
};
use bytes::{BufMut, Bytes, BytesMut};

trait Serialize {
    fn write(&self, buf: &mut BytesMut);
}

impl Serialize for &str {
    fn write(&self, buf: &mut BytesMut) {
        self.split('.').for_each(|l| {
            buf.put_u8(l.len() as u8);
            buf.put(l.as_bytes())
        });
        buf.put_u8(0);
    }
}

impl<T: Serialize> Serialize for Vec<T> {
    fn write(&self, buf: &mut BytesMut) {
        self.iter().for_each(|i| i.write(buf))
    }
}

impl Serialize for Header {
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

impl Serialize for Domain {
    fn write(&self, buf: &mut BytesMut) {
        self.name.as_str().write(buf);
        buf.put_u16(self.record as u16);
        buf.put_u16(self.class as u16);
    }
}

impl Serialize for Data {
    fn write(&self, buf: &mut BytesMut) {
        match self {
            Self::Ipv4(ip) => buf.put_slice(&ip.octets()),
        }
    }
}

impl Serialize for Route {
    fn write(&self, buf: &mut BytesMut) {
        self.domain().write(buf);
        buf.put_u32(self.ttl());
        buf.put_u16(self.data().len());
        self.data().write(buf);
    }
}

impl Message {
    pub fn flush(&self) -> Bytes {
        let mut buf = BytesMut::with_capacity(12);
        self.header().write(&mut buf);
        self.questions().write(&mut buf);
        self.answers().write(&mut buf);
        buf.freeze()
    }
}

#[cfg(test)]
mod tests {
    use crate::message::header::{OpCode, PacketId, Reserved};
    use std::net::Ipv4Addr;

    use super::*;

    #[test]
    fn test_write_domain() {
        let d = Domain::new_aa("google.com");
        let mut buf = BytesMut::new();
        d.write(&mut buf);

        let chunk: &[u8] = &[
            0x6, 0x67, 0x6f, 0x6f, 0x67, 0x6c, 0x65, 0x03, 0x63, 0x6f, 0x6d, 0, 0, 1, 0, 1,
        ];
        assert_eq!(buf.as_ref(), chunk);
    }

    #[test]
    fn test_write_header() {
        let h = Header {
            id: PacketId(34346),
            qr: QueryMode::Query,
            op_code: OpCode::no_error(),
            aa: Authoritative::Unowned,
            tc: Truncation::Complete,
            rd: Recursion::Enabled,
            ra: Recursion::Disabled,
            z: Reserved,
            r_code: OpCode::no_error(),
            qd_count: 1,
            an_count: 0,
            ar_count: 0,
            ns_count: 0,
        };

        let mut buf = BytesMut::new();
        h.write(&mut buf);

        let chunk: &[u8] = &[0x86, 0x2a, 0x01, 00, 00, 1, 00, 00, 00, 00, 00, 00];
        assert_eq!(buf.as_ref(), chunk);
    }

    #[test]
    fn test_write_answer() {
        let dn = Domain::new_aa("google.com");
        let d = Data::Ipv4(Ipv4Addr::new(0, 0, 0, 0));
        let a = Route::new(dn, 60, d);

        let mut buf = BytesMut::new();
        a.write(&mut buf);

        let chunk: &[u8] = &[
            0x6, 0x67, 0x6f, 0x6f, 0x67, 0x6c, 0x65, 0x03, 0x63, 0x6f, 0x6d, 0, 0, 1, 0, 1, 0, 0,
            0, 0x3c, 0, 4, 0, 0, 0, 0,
        ];
        assert_eq!(buf.as_ref(), chunk);
    }
}
