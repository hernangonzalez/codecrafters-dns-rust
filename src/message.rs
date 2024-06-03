pub mod header;
pub use header::Header;

#[derive(Clone, Copy, Debug)]
pub struct Message {
    pub header: Header,
}

impl Message {
    pub fn response(query: Message) -> Self {
        Self {
            header: Header::response(query.header.id),
        }
    }
}
