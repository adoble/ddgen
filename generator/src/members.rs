use bit_lang::BitSpec;
use genco::prelude::*;
use serde::Deserialize;
use std::collections::HashMap;

use crate::common_structure::CommonStructure;
use crate::field::Field;

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Members(HashMap<String, Field>);

impl Members {
    pub fn generate_serializations(
        &self,
        tokens: &mut Tokens<Rust>,
        struct_name: &str,
        common_structures: &HashMap<String, CommonStructure>,
    ) {
        // Generate a table that maps bitspecs to symbols. Note this is
        // only for the request/serialization as the response may have
        // different symbols
        let mut symbol_table: HashMap<BitSpec, String> = HashMap::new();

        for (name, field) in self.0.iter() {
            match field {
                Field::BitField { bit_spec, .. } => {
                    symbol_table.insert(bit_spec.clone(), name.to_string())
                }
                Field::Structure {
                    common_structure_name,
                    bit_spec,
                    ..
                } => symbol_table.insert(bit_spec.clone(), name.to_string()),
            };
        }

        let mut sorted_members: Vec<_> = self.to_vec();

        // Sort by fields, not by the name
        sorted_members.sort_by(|(_, field_a), (_, field_b)| field_a.cmp(field_b));

        let serialization_buffer_size = self.buffer_size(&common_structures);

        quote_in!(*tokens =>
            impl Serialize for $(struct_name) {
                fn serialize<const N: usize>(&self) -> (u8, [u8; N]) {
                  let mut data = [0u8; N];

                  $(for (name, field) in sorted_members => $(ref toks {field.generate_field_serialization(toks,  name,  &symbol_table)}) )

                  ($(serialization_buffer_size), data)
                }


            }
        );
    }

    pub fn generate_deserializations(
        &self,
        tokens: &mut Tokens<Rust>,
        struct_name: &str,

        common_structures: &HashMap<String, CommonStructure>,
    ) {
        // Generate a table that maps bitspecs to symbols. Note this is
        // only for the response/deserialization as the request may have
        // different symbols
        let mut symbol_table: HashMap<BitSpec, String> = HashMap::new();
        for (name, field) in self.iter() {
            match field {
                Field::BitField { bit_spec, .. } => {
                    symbol_table.insert(bit_spec.clone(), name.to_string())
                }
                Field::Structure {
                    common_structure_name,
                    bit_spec,
                    ..
                } => symbol_table.insert(bit_spec.clone(), name.to_string()),
            };
        }

        //let mut sorted_members: Vec<_> = members.iter().collect();
        let mut sorted_members: Vec<_> = self.to_vec();

        // Sort by fields, not by the name
        sorted_members.sort_by(|(_, field_a), (_, field_b)| field_a.cmp(field_b));

        quote_in!(*tokens=>
           impl Deserialize<$(struct_name)> for [u8] {

               fn deserialize(&self) -> Result<$(struct_name), DeviceError> { $['\r']

                    Ok($(struct_name) {$['\r']
                        $(for (name, field) in sorted_members => $(name): $(ref toks {field.generate_field_deserialization(toks,  name, &symbol_table)}) ) $['\r']
                    })$['\r']


               }$['\r']

           }$['\r']
        );
    }

    /// Calculates the max size in bytes of a set of members. This is required
    /// so that the buffers for the structures can be sized to cater for
    /// largest size.
    pub fn buffer_size(&self, common_structures: &HashMap<String, CommonStructure>) -> usize {
        let mut positions: HashMap<usize, usize> = HashMap::new();
        for f in self.fields() {
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
                    // if !positions.contains_key(&bit_range.start.index) {
                    //     positions.insert(bit_range.start.index, bit_range.max_size());
                    // }
                    //buffer_size += bit_range.max_size()
                }
                Field::Structure {
                    common_structure_name,
                    bit_spec,
                    ..
                } => {
                    let common_structure = common_structures.get(common_structure_name).unwrap();
                    positions
                        .entry(bit_spec.start.index)
                        .or_insert_with(|| common_structure.buffer_size());
                }
            }
        }

        positions.values().sum::<usize>()
    }

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
