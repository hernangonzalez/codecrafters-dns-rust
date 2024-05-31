use crate::message::{
    header::{OpCode, PacketId, Reserved},
    Header, Message,
};
use nom::{bits, combinator::map, number::complete::be_u16, sequence::tuple, IResult, Parser};

type ByteResult<'a, T> = IResult<&'a [u8], T>;
type BitInput<'a> = (&'a [u8], usize);
type BitResult<'a, T> = IResult<BitInput<'a>, T>;

fn take_packet_id(i: &[u8]) -> ByteResult<PacketId> {
    map(be_u16, PacketId).parse(i)
}

fn take_opcode(i: BitInput) -> BitResult<OpCode> {
    map(bits::complete::take(1u8), |bits: u8| OpCode(bits)).parse(i)
}

fn take_enum<T: From<u8>>(i: BitInput) -> BitResult<T> {
    map(bits::complete::take(1u8), |bits: u8| T::from(bits)).parse(i)
}

fn take_reserved(i: BitInput) -> BitResult<Reserved> {
    map(bits::complete::take(3u8), |_: u8| Reserved).parse(i)
}

fn parse_header(i: &[u8]) -> ByteResult<Header> {
    let (i, id) = take_packet_id(i)?;
    let (i, flags) = bits::bits(tuple((
        take_enum,
        take_opcode,
        take_enum,
        take_enum,
        take_enum,
        take_enum,
        take_reserved,
        take_opcode,
    )))
    .parse(i)?;

    let (i, qd_count) = be_u16(i)?;
    let (i, an_count) = be_u16(i)?;
    let (i, ar_count) = be_u16(i)?;
    let (i, ns_count) = be_u16(i)?;

    let header = Header {
        id,
        qr: flags.0,
        op_code: flags.1,
        aa: flags.2,
        tc: flags.3,
        ra: flags.4,
        rd: flags.5,
        z: flags.6,
        r_code: flags.7,
        qd_count,
        an_count,
        ar_count,
        ns_count,
    };

    ByteResult::Ok((i, header))
}

fn parse_message(i: &[u8]) -> ByteResult<Message> {
    map(parse_header, |header| Message { header }).parse(i)
}

impl TryFrom<&[u8]> for Message {
    type Error = anyhow::Error;
    fn try_from(buf: &[u8]) -> Result<Self, Self::Error> {
        parse_message(buf)
            .map(|i| i.1)
            .map_err(|e| anyhow::anyhow!("Could not read message in buffer: {}", e.to_string()))
    }
}
