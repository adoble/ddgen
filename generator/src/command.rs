use std::collections::HashMap;
use std::fs::File;
use std::path::Path;

use anyhow::Context;
use convert_case::{Case, Casing};
use genco::prelude::*;
use indexmap::IndexMap;
use serde::Deserialize;

use crate::doc_comment::DocComment;
use crate::field::Field;
use crate::output::output_file;
use bit_lang::BitSpec;

//use crate::generate::output_file; //TODO need to place this in it's own module

const VERSION: &str = env!("CARGO_PKG_VERSION");
// const PKG_NAME: &str = env!("CARGO_PKG_NAME");

/// Represents a command
// Using IndexMap so that the order of the membres is
// preserved between runs. Helps with code readablity and testing.
#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Command {
    opcode: u8,

    description: Option<String>,

    request: IndexMap<String, Field>,
    response: IndexMap<String, Field>,
}

impl Command {
    pub fn generate_command(&self, name: &str, out_path: &Path) -> anyhow::Result<()> {
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
            #![allow(unused_imports)]
            $(command_doc_comment)
            $(description_doc_comment)
            $(generated_doc_comment)


            use crate::deserialize::Deserialize;
            use crate::error::DeviceError;
            use crate::request::{RequestArray, RequestBit, RequestField, RequestWord};
            use crate::response::{ResponseArray, ResponseBit, ResponseField, ResponseWord};
            use crate::serialize::Serialize;
            use crate::types::*;


            #[derive(Debug, PartialEq)]
            pub struct $(&request_struct_name) {
                $(ref toks => self.generate_members(toks, &self.request))
            }

            $(ref toks => self.generate_serializations(toks, &request_struct_name, &self.request))

            #[derive(Debug, PartialEq)]
            pub struct $(&response_struct_name) {
                $(ref toks => self.generate_members(toks,  &self.response))
            }

            $(ref toks => self.generate_deserializations(toks, &response_struct_name,&self.response))



        );

        output_file(file, tokens)?;

        Ok(())
    }

    fn generate_members(&self, tokens: &mut Tokens<Rust>, members: &IndexMap<String, Field>) {
        // TODO
        // Sort the members
        //

        // Use a vec to sort the members
        let mut sorted_vec = Vec::new();
        for (name, field) in members {
            sorted_vec.push((name, field));
        }
        sorted_vec.sort();

        for entry in sorted_vec {
            entry.1.generate_struct_member(tokens, entry.0);
        }

        // for (name, field) in members {
        //     field.generate_struct_member(tokens, name);
        // }
    }

    fn generate_serializations(
        &self,
        tokens: &mut Tokens<Rust>,
        struct_name: &str,
        members: &IndexMap<String, Field>,
    ) {
        // Generate a table that maps bitspecs to symbols. Note this is
        // only for the request/serialization as the response may have
        // different symbols
        let mut symbol_table: HashMap<BitSpec, String> = HashMap::new();
        for (name, field) in members {
            if let Field::BitField { bit_spec, .. } = field {
                symbol_table.insert(bit_spec.clone(), name.to_string());
            } else {
                todo!("Handle structures");
            }
        }

        let serialization_buffer_size = self.buffer_size(members);

        quote_in!(*tokens =>
            impl Serialize for $(struct_name) {
                fn serialize<const N: usize>(&self) -> (u8, [u8; N]) {
                  let mut data = [0u8; N];

                  $(for (name, field) in members => $(ref toks {field.generate_field_serialization(toks,  name,  &symbol_table)}) )

                  ($(serialization_buffer_size), data)
                }


            }
        );
    }

    fn generate_deserializations(
        &self,
        tokens: &mut Tokens<Rust>,
        struct_name: &str,
        members: &IndexMap<String, Field>,
    ) {
        // Generate a table that maps bitspecs to symbols. Note this is
        // only for the response/deserialization as the request may have
        // different symbols
        let mut symbol_table: HashMap<BitSpec, String> = HashMap::new();
        for (name, field) in members {
            if let Field::BitField { bit_spec, .. } = field {
                symbol_table.insert(bit_spec.clone(), name.to_string());
            } else {
                todo!("Handle structures");
            }
        }

        quote_in!(*tokens=>
           impl Deserialize<$(struct_name)> for [u8] {

               fn deserialize(&self) -> Result<$(struct_name), DeviceError> {
                    Ok($(struct_name) {
                        $(for (name, field) in members => $(name): $(ref toks {field.generate_field_deserialization(toks,  name, &symbol_table)}) )
                    })
               }
           }
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

    pub fn buffer_size(&self, members: &IndexMap<String, Field>) -> usize {
        let mut positions: HashMap<usize, usize> = HashMap::new();
        for f in members.values() {
            // Implementing this with a hashmap as more than one bit spec can reference the
            // same position in the buffer. The key is the position and the value is the size.
            match f {
                Field::BitField { bit_spec: bit_range, .. } => {
                    positions.entry(bit_range.start.index).or_insert_with(|| bit_range.max_size());
                    // if !positions.contains_key(&bit_range.start.index) {
                    //     positions.insert(bit_range.start.index, bit_range.max_size());
                    // }
                    //buffer_size += bit_range.max_size()
                }
                Field::Structure {
                    //common_structure_name,
                    ..
                } => todo!(),
            }
        }

        positions.values().sum::<usize>()
    }
}
