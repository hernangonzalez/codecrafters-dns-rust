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

#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct OpCode(pub u8); // 4 bits

impl OpCode {
    pub fn no_error() -> Self {
        OpCode(0)
    }

    pub fn not_implemented() -> Self {
        OpCode(4)
    }
}

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
    pub rd: Recursion,
    pub ra: Recursion,
    pub z: Reserved,
    pub r_code: OpCode,
    pub qd_count: u16,
    pub an_count: u16,
    pub ar_count: u16,
    pub ns_count: u16,
}

impl Default for Header {
    fn default() -> Self {
        Self {
            id: PacketId(0),
            qr: QueryMode::Response,
            op_code: OpCode::default(),
            aa: Authoritative::Unowned,
            tc: Truncation::Complete,
            ra: Recursion::Disabled,
            rd: Recursion::Disabled,
            z: Reserved,
            r_code: OpCode::default(),
            qd_count: 0,
            an_count: 0,
            ar_count: 0,
            ns_count: 0,
        }
    }
}

impl Header {
    pub fn response(id: PacketId) -> Self {
        Self {
            id,
            ..Default::default()
        }
    }
}
