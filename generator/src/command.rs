use std::collections::HashMap;
use std::fs::File;
use std::path::Path;

use anyhow::Context;
use convert_case::{Case, Casing};
use genco::prelude::*;
use serde::Deserialize;

use crate::common_structure::CommonStructure;
use crate::doc_comment::DocComment;
use crate::members::Members;
use crate::output::output_file;

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
            use crate::request::{RequestArray, RequestBit, RequestField, RequestWord, RequestStruct};
            use crate::response::{ResponseArray, ResponseBit, ResponseField, ResponseWord};
            use crate::serialize::Serialize;
            use crate::types::*;

            $(for name in common_structures.keys() => use crate::$(name.to_lowercase())::$(name.to_case(Case::UpperCamel));)

            $['\n']
            #[derive(Debug, PartialEq)]$['\r']
            pub struct $(&request_struct_name) {$['\r']
                $(ref toks => self.request.generate_members(toks))$['\r']
            }
            $['\n']
            $(ref toks => self.request.generate_serializations(toks, &request_struct_name,  &common_structures))$['\r']

            $['\n']
            #[derive(Debug, PartialEq)]$['\r']
            pub struct $(&response_struct_name) {$['\r']
                $(ref toks => self.response.generate_members(toks))$['\r']
            }
            $['\n']
            $(ref toks => self.response.generate_deserializations(toks, &response_struct_name))$['\r']



        );

        output_file(file, tokens)?;

        Ok(())
    }
}
