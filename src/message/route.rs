use super::{data::Data, domain::Domain};

#[derive(Clone, Debug, PartialEq)]
pub struct Route {
    domain: Domain,
    ttl: u32,
    data: Data,
}

impl Route {
    pub fn new(domain: Domain, ttl: u32, data: Data) -> Self {
        Self { domain, ttl, data }
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::Ipv4Addr;

    #[test]
    fn test_answer_aa() {
        let dn = Domain::new_aa("hernan.rs");
        let d = Data::Ipv4(Ipv4Addr::new(1, 1, 1, 1));
        let a = Route::new(dn.clone(), 60, d);
        assert_eq!(a.domain, dn);
        assert_eq!(a.ttl(), 60);
        assert_eq!(a.data(), &d);
    }
}
