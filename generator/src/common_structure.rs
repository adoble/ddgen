use serde::Deserialize;

use convert_case::{Case, Casing};
use genco::prelude::*;

use std::collections::HashMap;

use crate::field::Field;

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct CommonStructure(HashMap<String, Field>);

impl CommonStructure {
    pub fn generate(&self, tokens: &mut Tokens<Rust>, name: String) {
        let struct_name = name.to_case(Case::UpperCamel);
        quote_in!(*tokens =>
            pub struct $struct_name {
                $(for (name, field) in &self.0 => $(ref toks {field.generate_struct_member(toks, name)}) )
            }
        );
    }
}
