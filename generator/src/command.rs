use std::collections::HashMap;
use std::fs::File;
use std::path::Path;

use anyhow::Context;
use convert_case::{Case, Casing};
use genco::prelude::*;
use serde::Deserialize;

use crate::common_structure::CommonStructure;
use crate::doc_comment::DocComment;
use crate::field::Field;
use crate::members::Members;
use crate::output::output_file;
use bit_lang::BitSpec;

//use crate::generate::output_file; //TODO need to place this in it's own module

const VERSION: &str = env!("CARGO_PKG_VERSION");
// const PKG_NAME: &str = env!("CARGO_PKG_NAME");

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Command {
    opcode: u8,

    description: Option<String>,

    // request: HashMap<String, Field>,
    // response: HashMap<String, Field>,
    request: Members,
    response: Members,
}

impl Command {
    pub fn generate_command(
        &self,
        name: &str,
        common_structures: &HashMap<String, CommonStructure>,
        out_path: &Path,
    ) -> anyhow::Result<()> {
        println!("Generating command files ...");

        let command_file_name = format!("{}.rs", name.to_lowercase());
        let target_path = out_path.join(command_file_name.clone());

        let file = File::create(target_path)
            .with_context(|| format!("Cannot open output file {}", command_file_name))?;

        let mut tokens = rust::Tokens::new();

        let request_struct_name = format!("{}Request", name.to_case(Case::UpperCamel));
        let response_struct_name = format!("{}Response", name.to_case(Case::UpperCamel));

        let command_doc_comment = DocComment::from_string(&format!("Command {}", name)).as_string();
        let description_doc_comment =
            DocComment::from_string(self.description.as_ref().unwrap_or(&String::new()))
                .as_string();
        let generated_doc_comment =
            DocComment::from_string(&format!("Generated with version {} of ddgen", VERSION))
                .as_string();

        // DEBUG
        quote_in!(tokens =>
            #![allow(unused_imports)]$['\n']
            $(command_doc_comment)$['\r']

            $(description_doc_comment)$['\r']

            $(generated_doc_comment)$['\n']


            use crate::deserialize::Deserialize;
            use crate::error::DeviceError;
            use crate::request::{RequestArray, RequestBit, RequestField, RequestWord};
            use crate::response::{ResponseArray, ResponseBit, ResponseField, ResponseWord};
            use crate::serialize::Serialize;
            use crate::types::*;

            $(for name in common_structures.keys() => use crate::$(name.to_lowercase())::$(name.to_case(Case::UpperCamel));)

            $['\n']
            #[derive(Debug, PartialEq)]$['\r']
            pub struct $(&request_struct_name) {$['\r']
                $(ref toks => self.generate_members(toks, &self.request))$['\r']
            }
            $['\n']
            $(ref toks => self.generate_serializations(toks, &request_struct_name, &self.request, &common_structures))$['\r']

            $['\n']
            #[derive(Debug, PartialEq)]$['\r']
            pub struct $(&response_struct_name) {$['\r']
                $(ref toks => self.generate_members(toks,  &self.response))$['\r']
            }
            $['\n']
            $(ref toks => self.generate_deserializations(toks, &response_struct_name,&self.response, &common_structures))$['\r']



        );

        output_file(file, tokens)?;

        Ok(())
    }

    fn generate_members(&self, tokens: &mut Tokens<Rust>, members: &Members) {
        // let mut sorted_members: Vec<_> = members.iter().collect();
        let mut sorted_members = members.to_vec();

        // Sort by fields, not by the name
        sorted_members.sort_by(|(_, field_a), (_, field_b)| field_a.cmp(field_b));

        for (name, field) in sorted_members {
            field.generate_struct_member(tokens, name);
        }
    }

    fn generate_serializations(
        &self,
        tokens: &mut Tokens<Rust>,
        struct_name: &str,
        members: &Members,
        common_structures: &HashMap<String, CommonStructure>,
    ) {
        // Generate a table that maps bitspecs to symbols. Note this is
        // only for the request/serialization as the response may have
        // different symbols
        let mut symbol_table: HashMap<BitSpec, String> = HashMap::new();

        for (name, field) in members.iter() {
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

        let mut sorted_members: Vec<_> = members.to_vec();

        // Sort by fields, not by the name
        sorted_members.sort_by(|(_, field_a), (_, field_b)| field_a.cmp(field_b));

        let serialization_buffer_size = self.buffer_size(members, &common_structures);

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

    fn generate_deserializations(
        &self,
        tokens: &mut Tokens<Rust>,
        struct_name: &str,
        members: &Members,
        common_structures: &HashMap<String, CommonStructure>,
    ) {
        // Generate a table that maps bitspecs to symbols. Note this is
        // only for the response/deserialization as the request may have
        // different symbols
        let mut symbol_table: HashMap<BitSpec, String> = HashMap::new();
        for (name, field) in members.iter() {
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
        let mut sorted_members: Vec<_> = members.to_vec();

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

    pub fn generate_enums(_tokens: &mut Tokens<Rust>, _name: &str) {
        // quote_in!(*tokens =>
        //     $['\n']
        //     /// Enumerations
        //     todo!()

        // );
        todo!();
    }

    /// Calculates the max size in bytes of a set of members. This is required
    /// so that the buffers for the structures can be sized to cater for
    /// largest size.
    pub fn buffer_size(
        &self,
        members: &Members,
        common_structures: &HashMap<String, CommonStructure>,
    ) -> usize {
        let mut positions: HashMap<usize, usize> = HashMap::new();
        for f in members.fields() {
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
}
