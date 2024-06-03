use crate::{
    message::{
        header::{Authoritative, QueryMode, Recursion, Truncation},
        Header,
    },
    Message,
};
use bytes::{BufMut, BytesMut};

impl Header {
    pub fn flush(&self, buf: BytesMut) -> BytesMut {
        let mut buf = buf;
        buf.put_u16(self.id.0);

        let mut flags = 0u8;
        if self.qr == QueryMode::Response {
            flags |= 0b1_0000000;
        }
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

        buf
    }
}

impl Message {
    pub fn flush(&self) -> BytesMut {
        let buf = BytesMut::with_capacity(12);
        self.header.flush(buf)
    }
}
