use std::net::Ipv4Addr;

use crate::message::{
    data::Data,
    domain::{Class, Domain, Record},
    header::{OpCode, PacketId, Reserved},
    route::Route,
    Header, Message,
};
use nom::{
    bits,
    bytes::complete::take,
    combinator::{map, map_res, peek},
    number::complete::{be_u16, be_u32, be_u8},
    sequence::tuple,
    IResult, Parser,
};

const STR_REF_MSB: u8 = 0b11000000;
const STR_REF_MSB_U16: u16 = (STR_REF_MSB as u16) << 8;

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

// TODO: Cache parsed domain parts
fn collect_domain_name<'a>(i: &'a [u8], buf: &'a [u8]) -> ByteResult<'a, Vec<String>> {
    let mut v = vec![];
    let mut i = i;

    loop {
        let (_, msb) = peek(be_u8).parse(i)?;
        if msb == 0 {
            i = &i[1..];
            break;
        }

        let is_ref = (msb & STR_REF_MSB) != 0;
        if is_ref {
            let pos;
            (i, pos) = be_u16(i)?;
            let pos = (pos & !STR_REF_MSB_U16) as usize;
            let j = &buf[pos..];
            let (_, mut other) = collect_domain_name(j, buf)?;
            v.append(&mut other);
            break;
        } else {
            let n;
            let s;
            (i, n) = be_u8(i)?;
            (i, s) = map_res(take(n), std::str::from_utf8).parse(i)?;
            v.push(s.to_string());
        }
    }
    Ok((i, v))
}

fn parse_domain_name<'a>(i: &'a [u8], buf: &'a [u8]) -> ByteResult<'a, String> {
    let (i, v) = collect_domain_name(i, buf)?;
    let n = v.join(".");
    Ok((i, n))
}

fn parse_record(i: &[u8]) -> ByteResult<Record> {
    map_res(be_u16, Record::try_from).parse(i)
}

fn parse_class(i: &[u8]) -> ByteResult<Class> {
    map_res(be_u16, Class::try_from).parse(i)
}

fn parse_domain<'a>(i: &'a [u8], buf: &'a [u8]) -> ByteResult<'a, Domain> {
    let (i, name) = parse_domain_name(i, buf)?;
    let (i, record) = parse_record(i)?;
    let (i, class) = parse_class(i)?;
    let q = Domain {
        name,
        record,
        class,
    };
    Ok((i, q))
}

fn parse_questions<'a>(i: &'a [u8], c: u16, buf: &'a [u8]) -> ByteResult<'a, Vec<Domain>> {
    (0..c).try_fold((i, vec![]), |(i, mut v), _| {
        let (i, q) = parse_domain(i, buf)?;
        v.push(q);
        Ok((i, v))
    })
}

fn parse_answers<'a>(i: &'a [u8], c: u16, buf: &'a [u8]) -> ByteResult<'a, Vec<Route>> {
    (0..c).try_fold((i, vec![]), |(i, mut v), _| {
        let (i, domain) = parse_domain(i, buf)?;
        let (i, ttl) = be_u32(i)?;
        let (i, len) = be_u16(i)?;
        let (i, ip) = take(len).parse(i)?;
        let ip = Ipv4Addr::new(ip[0], ip[1], ip[2], ip[3]);
        let data = Data::Ipv4(ip);
        let route = Route::new(domain, ttl, data);
        v.push(route);
        Ok((i, v))
    })
}

// TODO: Propagate Message error instead of panic.
fn parse_message(buf: &[u8]) -> ByteResult<Message> {
    let (i, header) = parse_header(buf)?;
    let (i, questions) = parse_questions(i, header.qd_count, buf)?;
    let (i, answers) = parse_answers(i, header.an_count, buf)?;
    let mut msg = Message::new(header);
    msg.set_questions(questions)
        .expect("Could not build questions");
    msg.set_answers(answers).expect("Could not build answers");
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
        assert_eq!(res.0.len(), 0);
        assert_eq!(res.1.id, PacketId(34346));
        assert_eq!(res.1.rd, Recursion::Enabled);
    }

    #[test]
    fn test_parse_domain() {
        let buf: &[u8] = &[
            0x6, 0x67, 0x6f, 0x6f, 0x67, 0x6c, 0x65, 0x03, 0x63, 0x6f, 0x6d, 0, 0, 1, 0, 1,
        ];

        let res = parse_domain(buf, buf).unwrap();

        assert_eq!(res.0.len(), 0);
        assert_eq!(res.1.name, "google.com");
        assert_eq!(res.1.record, Record::AA);
        assert_eq!(res.1.class, Class::IN);
    }

    #[test]
    fn test_parse_compressed_questions() {
        let data = [
            21, 51, 1, 0, 0, 2, 0, 0, 0, 0, 0, 0, 3, 97, 98, 99, 17, 108, 111, 110, 103, 97, 115,
            115, 100, 111, 109, 97, 105, 110, 110, 97, 109, 101, 3, 99, 111, 109, 0, 0, 1, 0, 1, 3,
            100, 101, 102, 192, 16, 0, 1, 0, 1,
        ];

        let (i, msg) = parse_message(data.as_ref()).unwrap();

        assert_eq!(i.len(), 0);
        assert_eq!(msg.questions().len(), 2);
        assert_eq!(
            msg.questions()[0],
            Domain::new_aa("abc.longassdomainname.com")
        );
        assert_eq!(
            msg.questions()[1],
            Domain::new_aa("def.longassdomainname.com")
        );
    }

    #[test]
    fn test_parse_response() {
        let data = std::fs::read("response_packet.bin").unwrap();
        assert!(!data.is_empty());

        let (i, msg) = parse_message(&data).expect("message");

        assert!(i.is_empty());
        assert_eq!(msg.questions().len(), 1);
        assert_eq!(msg.answers().len(), 4);
    }
}
