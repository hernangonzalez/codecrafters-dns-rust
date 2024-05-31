use crate::Message;
use anyhow::Result;
use bytes::BytesMut;

impl Message {
    pub fn flush(&self) -> Result<BytesMut> {
        todo!()
    }
}
