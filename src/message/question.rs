use super::domain::Domain;
use anyhow::Result;
use std::ops::Deref;

#[derive(Clone, Debug, Default)]
pub struct Questions(Vec<Domain>);

impl Deref for Questions {
    type Target = Vec<Domain>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl TryFrom<Vec<Domain>> for Questions {
    type Error = anyhow::Error;
    fn try_from(qs: Vec<Domain>) -> Result<Self> {
        anyhow::ensure!(
            qs.len() <= std::u16::MAX as usize,
            "Exceed supported number questions: {}",
            qs.len()
        );
        Ok(Questions(qs))
    }
}
