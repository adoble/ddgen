use crate::field::Field;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Members(HashMap<String, Field>);

impl Members {
    pub fn to_vec(&self) -> Vec<(&String, &Field)> {
        let v = self.0.iter().collect();
        v
    }

    pub fn fields(&self) -> impl Iterator<Item = &Field> {
        self.0.values()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&String, &Field)> {
        self.0.iter()
    }
}
