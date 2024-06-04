pub mod header;
pub use header::Header;
use header::QueryMode;
use std::ops::Deref;

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

#[derive(Clone, Debug)]
pub struct Message {
    header: Header,
    questions: Questions,
}

impl Message {
    pub fn new(header: Header, questions: Vec<Question>) -> Self {
        Self {
            header,
            questions: Questions(questions),
        }
    }

    pub fn new_response(query: Message) -> Self {
        Self {
            header: Header::response(query.header.id),
            questions: Questions::default(),
        }
    }
}

impl Message {
    pub fn is_query(&self) -> bool {
        self.header.qr == QueryMode::Query
    }

    pub fn questions(&self) -> &Questions {
        &self.questions
    }

    pub fn set_questions(&mut self, qs: Vec<Question>) {
        if qs.len() > std::u16::MAX as usize {
            panic!("Number of items exceeds supported len: {}", qs.len());
        }
        self.header.qd_count = qs.len() as u16;
        self.questions = Questions(qs);
    }

    pub fn header(&self) -> &Header {
        &self.header
    }
}
