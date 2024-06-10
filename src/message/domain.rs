use anyhow::Result;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Record {
    AA = 1,
}

impl TryFrom<u16> for Record {
    type Error = anyhow::Error;
    fn try_from(value: u16) -> Result<Self> {
        match value {
            x if x == Self::AA as u16 => Ok(Self::AA),
            _ => Err(anyhow::anyhow!("Not a record: {value}")),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Class {
    IN = 1,
}

impl TryFrom<u16> for Class {
    type Error = anyhow::Error;
    fn try_from(value: u16) -> Result<Self> {
        match value {
            x if x == Self::IN as u16 => Ok(Self::IN),
            _ => Err(anyhow::anyhow!("Not a class: {value}")),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Domain {
    pub name: String,
    pub record: Record,
    pub class: Class,
}

#[cfg(test)]
impl Domain {
    pub fn new_aa(name: &str) -> Self {
        Self {
            name: name.to_string(),
            record: Record::AA,
            class: Class::IN,
        }
    }
}
