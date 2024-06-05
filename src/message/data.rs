use std::net::Ipv4Addr;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Data {
    Ipv4(Ipv4Addr),
}

impl Data {
    pub fn len(&self) -> u16 {
        match self {
            Self::Ipv4(ip) => ip.octets().len() as u16,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_len() {
        let d = Data::Ipv4(Ipv4Addr::new(1, 1, 1, 1));
        assert_eq!(d.len(), 4);
    }
}
