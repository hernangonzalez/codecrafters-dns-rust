#[allow(dead_code)]
#[derive(Clone, Copy, Debug)]
struct PacketId(u16);

#[allow(dead_code)]
#[derive(Clone, Copy, Debug)]
enum QueryMode {
    Query,
    Response,
}

#[allow(dead_code)]
#[derive(Clone, Copy, Debug)]
struct OpCode(u8); // 4 bits

#[allow(dead_code)]
#[derive(Clone, Copy, Debug)]
struct Authoritative(bool);

#[allow(dead_code)]
#[derive(Clone, Copy, Debug)]
struct Truncated(bool);

#[allow(dead_code)]
#[derive(Clone, Copy, Debug)]
struct Recursive(bool);

#[allow(dead_code)]
#[derive(Clone, Copy, Debug)]
struct Reserved; // 3 bits

#[allow(dead_code)]
#[derive(Clone, Copy, Debug)]
struct ResCode(u8); // 4 bits

#[allow(dead_code)]
#[derive(Clone, Copy, Debug)]
struct Header {
    id: PacketId,
    qr: QueryMode,
    op_code: OpCode,
    aa: Authoritative,
    tc: Truncated,
    red: Recursive,
    z: Reserved,
    r_code: ResCode,
    qd_count: u16,
    an_count: u16,
    ns_count: u16,
    ar_count: u16,
}

#[allow(dead_code)]
#[derive(Clone, Copy, Debug)]
struct Message {
    header: Header,
}
