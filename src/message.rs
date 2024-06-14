pub mod data;
pub mod domain;
pub mod header;
pub mod route;
use anyhow::Result;
use domain::Domain;
pub use header::Header;
use header::{OpCode, QueryMode};
use route::Route;

#[derive(Clone, Debug)]
pub struct Message {
    header: Header,
    questions: Vec<Domain>,
    answers: Vec<Route>,
}

impl Message {
    pub fn new(header: Header) -> Self {
        let mut header = header;
        header.qd_count = 0;
        header.an_count = 0;
        Self {
            header,
            questions: Default::default(),
            answers: Default::default(),
        }
    }

    pub fn new_response(query: &Message) -> Self {
        let mut header = Header::response(query.header.id);
        header.op_code = query.header.op_code;
        header.rd = query.header.rd;
        header.r_code = if query.header.op_code == OpCode(0) {
            OpCode::no_error()
        } else {
            OpCode::not_implemented()
        };
        header.qd_count = query.header.qd_count;
        Self {
            header,
            questions: query.questions.clone(),
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

    pub fn questions(&self) -> &Vec<Domain> {
        &self.questions
    }

    pub fn set_questions(&mut self, qs: Vec<Domain>) -> Result<()> {
        anyhow::ensure!(
            qs.len() <= u16::MAX as usize,
            "Exceed supported number of domains: {}",
            qs.len()
        );

        self.header.qd_count = qs.len() as u16;
        self.questions = qs;
        Ok(())
    }

    pub fn answers(&self) -> &Vec<Route> {
        &self.answers
    }

    pub fn set_answers(&mut self, ans: Vec<Route>) -> Result<()> {
        anyhow::ensure!(
            ans.len() <= u16::MAX as usize,
            "Exceed supported max number of routes: {}",
            ans.len()
        );

        self.header.an_count = ans.len() as u16;
        self.answers = ans;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain::Domain;
    use header::PacketId;
    use route::Route;
    use std::net::Ipv4Addr;

    #[test]
    fn test_message_empty() {
        let h = Header {
            id: PacketId(42),
            ..Default::default()
        };

        let msg = Message::new(h);

        assert_eq!(msg.header.id, PacketId(42));
        assert_eq!(msg.header.an_count, 0);
        assert_eq!(msg.header.qd_count, 0);
    }

    #[test]
    fn test_message_response() {
        let h = Header {
            id: PacketId(42),
            ..Default::default()
        };
        let mut query = Message::new(h);
        let q = Domain::new_aa("hernan.rs");
        let qs = vec![q.clone()];
        query.set_questions(qs).unwrap();

        let msg = Message::new_response(&query);

        assert_eq!(msg.header().id, query.header().id);
        assert_eq!(msg.header().rd, query.header().rd);
        assert_eq!(msg.header().qd_count, 1);
        assert!(msg.questions().contains(&q));
    }

    #[test]
    fn test_message_answers() {
        let h = Header {
            id: PacketId(6),
            ..Default::default()
        };
        let d = Domain::new_aa("hernan.rs");
        let a = Route::new(d, 60, data::Data::Ipv4(Ipv4Addr::new(0, 0, 0, 0)));
        let ans = vec![a.clone()];

        let mut msg = Message::new(h);
        msg.set_answers(ans).unwrap();

        assert_eq!(msg.header().an_count, 1);
        assert!(msg.answers().contains(&a));
    }
}
