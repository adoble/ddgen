use serde::Deserialize;
use std::collections::HashMap;

use convert_case::{Case, Casing};
use genco::prelude::*;

use crate::{field::Field, members::Members};

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct CommonStructure(Members);

impl CommonStructure {
    pub fn generate(
        &self,
        tokens: &mut Tokens<Rust>,
        name: String,
        common_structures: &HashMap<String, CommonStructure>,
    ) {
        let struct_name = name.to_case(Case::UpperCamel);
        quote_in!(*tokens =>
            #[derive(Debug, PartialEq, Copy, Clone)]
            pub struct $(struct_name.clone()) {
                $(for (name, field) in self.0.iter() => $(ref toks {field.generate_struct_member(toks, name)}) )
            }

            $(ref toks => self.0.generate_serializations(toks, &struct_name, &common_structures))$['\r']

            $(ref toks => self.0.generate_deserializations(toks, &struct_name))$['\r']
        );
    }

    // pub fn generate_serializations(
    //     &self,
    //     tokens: &mut Tokens<Rust>,
    //     common_structure_name: String,
    //     common_structures: &HashMap<String, CommonStructure>,
    // ) {
    //     // Note: Common structure cannot contain other common structures

    //     quote_in!(*tokens =>

    //        $(ref toks => self.0.generate_serializations(toks, &common_structure_name, &common_structures))$['\r']

    //     );
    // }

    /// Determine how many bytes this structure would need.
    /// Note: Common Structures cannot contain variable fields.
    // In the future this should
    // return a Vec of tuples - `Vec<(usize, Option<String>)>` - each containing the
    // fixed size of the repeating elements and an optional String with the symbolic name
    // of the mutiplieing field
    pub fn size(&self) -> usize {
        let size = self.0.size();
        assert!(
            size.1.len() == 0,
            "Fatal Error: Common structures should not contain varaible fields."
        );
        size.0
    }

    /// Calculates the size in bytes required to hold a common structure.
    /// Common structures cannot contain other common structures
    #[allow(dead_code)]
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
