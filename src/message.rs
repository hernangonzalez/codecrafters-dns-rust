pub mod header;
pub use header::Header;

#[derive(Clone, Copy, Debug)]
pub struct Message {
    pub header: Header,
}
