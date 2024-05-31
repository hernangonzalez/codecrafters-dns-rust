#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PacketId(pub u16);

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum QueryMode {
    Query = 0,
    Response = 1,
}

impl From<u8> for QueryMode {
    fn from(value: u8) -> Self {
        if value == Self::Response as u8 {
            Self::Response
        } else {
            Self::Query
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OpCode(pub u8); // 4 bits

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Authoritative {
    Owned = 1,
    Unowned = 0,
}

impl From<u8> for Authoritative {
    fn from(value: u8) -> Self {
        if value == Self::Owned as u8 {
            Self::Owned
        } else {
            Self::Unowned
        }
    }
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Truncation {
    Complete = 0,
    Truncated = 1,
}

impl From<u8> for Truncation {
    fn from(value: u8) -> Self {
        if value == Self::Truncated as u8 {
            Self::Truncated
        } else {
            Self::Complete
        }
    }
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Recursion {
    Disabled = 0,
    Enabled = 1,
}

impl From<u8> for Recursion {
    fn from(value: u8) -> Self {
        if value == Self::Enabled as u8 {
            Self::Enabled
        } else {
            Self::Disabled
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Reserved; // 3 bits

#[derive(Clone, Copy, Debug)]
pub struct Header {
    pub id: PacketId,
    pub qr: QueryMode,
    pub op_code: OpCode,
    pub aa: Authoritative,
    pub tc: Truncation,
    pub ra: Recursion,
    pub rd: Recursion,
    pub z: Reserved,
    pub r_code: OpCode,
    pub qd_count: u16,
    pub an_count: u16,
    pub ar_count: u16,
    pub ns_count: u16,
}
