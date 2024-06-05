use crate::message::{
    domain::{Class, Domain, Label, Record},
    header::{OpCode, PacketId, Reserved},
    Header, Message,
};
use nom::{
    bits,
    bytes::complete::{take, take_till1},
    combinator::{map, map_res, peek},
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

fn parse_label(i: &[u8]) -> ByteResult<Label> {
    let (i, msb) = peek(be_u16).parse(i)?;
    let is_ref = (msb & 0b11000000_00000000) != 0;
    if is_ref {
        let pos = msb & !0b11000000_00000000;
        let i = &i[2..];
        Ok((i, Label::Ref(pos)))
    } else {
        map(parse_string, Label::Domain).parse(i)
    }
}

fn parse_record(i: &[u8]) -> ByteResult<Record> {
    map_res(be_u16, Record::try_from).parse(i)
}

fn parse_class(i: &[u8]) -> ByteResult<Class> {
    map_res(be_u16, Class::try_from).parse(i)
}

fn parse_question(i: &[u8]) -> ByteResult<Domain> {
    let (i, name) = parse_label(i)?;
    let (i, record) = parse_record(i)?;
    let (i, class) = parse_class(i)?;
    let q = Domain {
        name,
        record,
        class,
    };
    Ok((i, q))
}

fn parse_questions(i: &[u8], c: u16) -> ByteResult<Vec<Domain>> {
    (0..c).try_fold((i, vec![]), |(i, mut v), _| {
        let (i, q) = parse_question(i)?;
        v.push(q);
        Ok((i, v))
    })
}

// TODO: Propagate Message error instead of panic.
fn parse_message(i: &[u8]) -> ByteResult<Message> {
    let (i, header) = parse_header(i)?;
    let (i, questions) = parse_questions(i, header.qd_count)?;
    let mut msg = Message::new(header);
    let questions = questions.try_into().expect("Could not build questions");
    msg.set_questions(questions);
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::message::header::Recursion;

    #[test]
    fn test_take_packet_id() {
        let res = take_packet_id(&[0, 6]);
        assert_eq!(res.unwrap().1, PacketId(6));

        let res = take_packet_id(&[0, 6, 0]).unwrap();
        assert_eq!(res.1, PacketId(6));
        assert_eq!(res.0.len(), 1);

        let res = take_packet_id(&[11]);
        assert!(res.is_err());
    }

    #[test]
    fn test_parse_header() {
        let buf: &[u8] = &[0x86, 0x2a, 0x01, 0x20, 00, 1, 00, 00, 00, 00, 00, 00];

        let res = parse_header(buf).unwrap();
        dbg!(res.1);

        assert_eq!(res.0.len(), 0);
        assert_eq!(res.1.id, PacketId(34346));
        assert_eq!(res.1.rd, Recursion::Enabled);
    }

    #[test]
    fn test_parse_question() {
        let buf: &[u8] = &[
            0x6, 0x67, 0x6f, 0x6f, 0x67, 0x6c, 0x65, 0x03, 0x63, 0x6f, 0x6d, 0, 0, 1, 0, 1,
        ];

        let res = parse_question(buf).unwrap();

        assert_eq!(res.0.len(), 0);
        assert_eq!(res.1.name, Label::Domain("google.com".into()));
        assert_eq!(res.1.record, Record::AA);
        assert_eq!(res.1.class, Class::IN);
    }

    #[test]
    fn test_parse_label_ref() {
        let data: [u8; 0x06] = [0xC0, 0x0C, 0x00, 0x01, 0x00, 0x01];
        let (_, l) = parse_label(&data).unwrap();
        dbg!(l);
    }

    #[test]
    fn test_parse_label_domain() {
        let data: [u8; 0x0C] = [
            0x06, 0x67, 0x6F, 0x6F, 0x67, 0x6C, 0x65, 0x03, 0x63, 0x6F, 0x6D, 0x00,
        ];
        let (_, l) = parse_label(&data).unwrap();
        dbg!(l);
    }

    #[test]
    fn test_parse_message() {
        let buf = std::fs::read("query_packet.bin").unwrap();

        let h = parse_header(buf.as_ref()).unwrap();
        dbg!(h);

        let msg = Message::try_from(buf.as_ref()).unwrap();

        dbg!(msg);
    }
}
