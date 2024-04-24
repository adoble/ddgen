use serde::Deserialize;

use convert_case::{Case, Casing};
use genco::prelude::*;

use std::collections::HashMap;

use crate::{field::Field, members::Members};

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct CommonStructure(Members);

impl CommonStructure {
    pub fn generate(&self, tokens: &mut Tokens<Rust>, name: String) {
        let struct_name = name.to_case(Case::UpperCamel);
        quote_in!(*tokens =>
            #[derive(Debug, PartialEq)]
            pub struct $struct_name {
                $(for (name, field) in self.0.iter() => $(ref toks {field.generate_struct_member(toks, name)}) )
            }
        );
    }

    pub fn generate_serializations(
        &self,
        tokens: &mut Tokens<Rust>,
        common_structure_name: String,
        common_structures: &HashMap<String, CommonStructure>,
    ) {
        // Note: Common structure cannot contain other common structures

        quote_in!(*tokens =>

           $(ref toks => self.0.generate_serializations(toks, &common_structure_name, &common_structures))$['\r']

        );
    }

    /// Calculates the size in bytes required to hold a common structure.
    /// Common structures cannot contain other common structures
    // TODO this is repeated in Command.buffer_size()
    pub fn buffer_size(&self) -> usize {
        let mut positions: HashMap<usize, usize> = HashMap::new();
        for f in self.0.fields() {
            // Implementing this with a hashmap as more than one bit spec can reference the
            // same position in the buffer. The key is the position and the value is the size.
            match f {
                Field::BitField {
                    bit_spec: bit_range,
                    ..
                } => {
                    positions
                        .entry(bit_range.start.index)
                        .or_insert_with(|| bit_range.max_size());
                }
                Field::Structure {
                    common_structure_name,
                    ..
                } => {
                    println!(
                        "A common structure cannot contain  another common structure ({})!",
                        common_structure_name
                    );
                    // TODO either make this impossible or provide better error handling!
                }
            }
        }

        positions.values().sum::<usize>()
    }
}
