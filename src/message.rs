pub mod header;
use anyhow::Result;
pub use header::Header;
use header::{OpCode, QueryMode};
use std::{net::Ipv4Addr, ops::Deref};

#[repr(u16)]
#[derive(Clone, Copy, Debug)]
pub enum Record {
    AA = 1,
}

#[derive(Clone, Copy, Debug)]
pub enum Class {
    IN = 1,
}

#[derive(Clone, Debug)]
pub struct Question {
    pub name: String,
    pub record: Record,
    pub class: Class,
}

impl Question {
    pub fn new_aa(name: &str) -> Self {
        Self {
            name: name.to_string(),
            record: Record::AA,
            class: Class::IN,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct Questions(Vec<Question>);

impl Deref for Questions {
    type Target = Vec<Question>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl TryFrom<Vec<Question>> for Questions {
    type Error = anyhow::Error;
    fn try_from(qs: Vec<Question>) -> Result<Self> {
        anyhow::ensure!(
            qs.len() <= std::u16::MAX as usize,
            "Exceed supported number questions: {}",
            qs.len()
        );
        Ok(Questions(qs))
    }
}

#[derive(Clone, Debug)]
pub enum Data {
    Ipv4(Ipv4Addr),
}

impl Data {
    fn len(&self) -> u16 {
        match self {
            Self::Ipv4(ip) => ip.octets().len() as u16,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Answer {
    name: String,
    record: Record,
    class: Class,
    ttl: u32,
    length: u16,
    data: Data,
}

impl Answer {
    pub fn new_aa(name: String, data: Data) -> Self {
        Self {
            name,
            record: Record::AA,
            class: Class::IN,
            ttl: 60,
            length: data.len(),
            data,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn record(&self) -> Record {
        self.record
    }

    pub fn class(&self) -> Class {
        self.class
    }

    pub fn ttl(&self) -> u32 {
        self.ttl
    }

    pub fn len(&self) -> u16 {
        self.length
    }

    pub fn data(&self) -> &Data {
        &self.data
    }
}

#[derive(Clone, Debug, Default)]
pub struct Answers(Vec<Answer>);

impl Deref for Answers {
    type Target = Vec<Answer>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl TryFrom<Vec<Answer>> for Answers {
    type Error = anyhow::Error;
    fn try_from(qs: Vec<Answer>) -> Result<Self> {
        anyhow::ensure!(
            qs.len() <= std::u16::MAX as usize,
            "Exceed supported number of Answers: {}",
            qs.len()
        );
        Ok(Answers(qs))
    }
}

#[derive(Clone, Debug)]
pub struct Message {
    header: Header,
    questions: Questions,
    answers: Answers,
}

impl Message {
    pub fn new_query(header: Header, questions: Vec<Question>) -> Self {
        Self {
            header,
            questions: Questions(questions),
            answers: Default::default(),
        }
    }

    pub fn new_response(query: Message) -> Self {
        let mut header = Header::response(query.header.id);
        header.op_code = query.header.op_code;
        header.rd = query.header.rd;
        header.r_code = if query.header.op_code == OpCode(0) {
            OpCode::no_error()
        } else {
            OpCode::not_implemented()
        };
        Self {
            header,
            questions: Default::default(),
            answers: Default::default(),
        }
    }
}

impl Message {
    pub fn header(&self) -> &Header {
        &self.header
    }

    pub fn is_query(&self) -> bool {
        self.header.qr == QueryMode::Query
    }

    pub fn questions(&self) -> &Questions {
        &self.questions
    }

    pub fn set_questions(&mut self, qs: Questions) {
        self.header.qd_count = qs.len() as u16;
        self.questions = qs;
    }

    pub fn answers(&self) -> &Answers {
        &self.answers
    }

    pub fn set_answers(&mut self, ans: Answers) {
        self.header.an_count = ans.len() as u16;
        self.answers = ans;
    }
}
