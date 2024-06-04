use crate::message::{
    header::{OpCode, PacketId, Reserved},
    Class, Header, Message, Question, Record,
};
use nom::{
    bits,
    bytes::complete::{take, take_till1},
    combinator::{map, map_res},
    number::{self, complete::be_u16},
    sequence::tuple,
    IResult, Parser,
};

type ByteResult<'a, T> = IResult<&'a [u8], T>;
type BitInput<'a> = (&'a [u8], usize);
type BitResult<'a, T> = IResult<BitInput<'a>, T>;

fn take_packet_id(i: &[u8]) -> ByteResult<PacketId> {
    map(be_u16, PacketId).parse(i)
}

fn take_opcode(i: BitInput) -> BitResult<OpCode> {
    map(bits::complete::take(4u8), |bits: u8| OpCode(bits)).parse(i)
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
        rd: flags.4,
        ra: flags.5,
        z: flags.6,
        r_code: flags.7,
        qd_count,
        an_count,
        ar_count,
        ns_count,
    };

    ByteResult::Ok((i, header))
}

fn parse_string(i: &[u8]) -> ByteResult<String> {
    let (i, mut buf) = take_till1(|b| b == 0).parse(i)?;
    let (i, _) = take(1u8).parse(i)?;
    let mut parts: Vec<String> = vec![];
    while !buf.is_empty() {
        let count;
        let chunk;
        (buf, count) = number::complete::u8(buf)?;
        (buf, chunk) = map_res(take(count), std::str::from_utf8).parse(buf)?;
        parts.push(chunk.to_string());
    }
    let s = parts.join(".");
    Ok((i, s))
}

fn parse_record(i: &[u8]) -> ByteResult<Record> {
    map_res(be_u16, Record::try_from).parse(i)
}

fn parse_class(i: &[u8]) -> ByteResult<Class> {
    map_res(be_u16, Class::try_from).parse(i)
}

fn parse_question(i: &[u8]) -> ByteResult<Question> {
    let (i, name) = parse_string(i)?;
    let (i, record) = parse_record(i)?;
    let (i, class) = parse_class(i)?;
    let q = Question {
        name,
        record,
        class,
    };
    Ok((i, q))
}

fn parse_questions(i: &[u8], c: u16) -> ByteResult<Vec<Question>> {
    (0..c).try_fold((i, vec![]), |(i, mut v), _| {
        let (i, q) = parse_question(i)?;
        v.push(q);
        Ok((i, v))
    })
}

fn parse_message(i: &[u8]) -> ByteResult<Message> {
    let (i, header) = parse_header(i)?;
    let (i, questions) = parse_questions(i, header.qd_count)?;
    let msg = Message::new_query(header, questions);
    Ok((i, msg))
}

impl TryFrom<&[u8]> for Message {
    type Error = anyhow::Error;
    fn try_from(buf: &[u8]) -> Result<Self, Self::Error> {
        parse_message(buf)
            .map(|i| i.1)
            .map_err(|e| anyhow::anyhow!("Could not read message in buffer: {}", e.to_string()))
    }
}
