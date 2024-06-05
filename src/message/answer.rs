use super::{data::Data, domain::Domain};
use anyhow::Result;
use std::ops::Deref;

#[derive(Clone, Debug, PartialEq)]
pub struct Answer {
    domain: Domain,
    ttl: u32,
    data: Data,
}

impl Answer {
    pub fn new(domain: Domain, data: Data) -> Self {
        Self {
            domain,
            ttl: 60,
            data,
        }
    }

    pub fn domain(&self) -> &Domain {
        &self.domain
    }

    pub fn ttl(&self) -> u32 {
        self.ttl
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::Ipv4Addr;

    #[test]
    fn test_answer_aa() {
        let dn = Domain::new_aa("hernan.rs");
        let d = Data::Ipv4(Ipv4Addr::new(1, 1, 1, 1));
        let a = Answer::new(dn.clone(), d);
        assert_eq!(a.domain, dn);
        assert_eq!(a.ttl(), 60);
        assert_eq!(a.data(), &d);
    }
}
