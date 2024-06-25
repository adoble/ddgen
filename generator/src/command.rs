use std::collections::HashMap;
use std::fs::File;
use std::path::Path;

use anyhow::Context;
use convert_case::{Case, Casing};
//use convert_case::{Case, Casing};
use genco::prelude::*;
use serde::Deserialize;

use crate::common_structure::CommonStructure;
use crate::doc_comment::DocComment;
use crate::flow_control::FlowControl;
use crate::members::Members;
use crate::naming::{CommandName, RequestStructName, ResponseStructName};
use crate::output::output_file;

const VERSION: &str = env!("CARGO_PKG_VERSION");
// const PKG_NAME: &str = env!("CARGO_PKG_NAME");

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Command {
    opcode: u8,

    description: Option<String>,

    #[serde(default)]
    flow_control: FlowControl,

    request: Members,
    response: Members,
}

impl Command {
    pub fn generate_command(
        &self,
        command_name: &CommandName,
        common_structures: &HashMap<String, CommonStructure>,
        out_path: &Path,
    ) -> anyhow::Result<()> {
        println!("Generating command file for {command_name}");

        let command_file_name = command_name.to_file_name();
        let target_path = out_path.join(command_file_name.clone());

        let file = File::create(target_path)
            .with_context(|| format!("Cannot open output file {}", command_file_name))?;

        let mut tokens = rust::Tokens::new();

        // let command_name = CommandName::from(command_name);
        let request_struct_name = RequestStructName::from(command_name);
        let response_struct_name = ResponseStructName::from(command_name);

        // let request_struct_name = format!("{}Request", name.to_case(Case::UpperCamel));
        // let response_struct_name = format!("{}Response", name.to_case(Case::UpperCamel));

        let command_doc_comment =
            DocComment::from_string(&format!("Command {}", command_name)).as_string();
        let description_doc_comment =
            DocComment::from_string(self.description.as_ref().unwrap_or(&String::new()))
                .as_string();
        let generated_doc_comment =
            DocComment::from_string(&format!("Generated with version {} of ddgen", VERSION))
                .as_string();

        // DEBUG
        quote_in!(tokens =>
            #![allow(unused_imports)]$['\n']
            #![allow(clippy::unnecessary_cast)]$['\n']
            $(command_doc_comment)$['\r']

            $(description_doc_comment)$['\r']

            $(generated_doc_comment)$['\n']

            use embedded_hal::spi::SpiDevice;

            use crate::command::Command;
            use crate::deserialize::Deserialize;
            use crate::error::DeviceError;

            use crate::request::{RequestArray, RequestBit, RequestField, RequestWord, RequestStruct};
            use crate::response::{ResponseArray, ResponseBit, ResponseField, ResponseWord};
            use crate::serialize::Serialize;
            use crate::types::*;

            use crate::transmit::Transmit;

            $(for name in common_structures.keys() => use crate::$(name.to_lowercase())::$(name.to_case(Case::UpperCamel));)

            $(for name in self.providers() => use crate::$(name.to_case(Case::Snake))::$(name.to_case(Case::UpperCamel));)


            $['\n']
            #[derive(Debug, PartialEq)]$['\r']
            pub struct $(request_struct_name.clone()) {$['\r']
                $(ref toks => self.request.generate_members(toks))$['\r']
            }
            $['\n']
            $(ref toks => self.request.generate_serializations(toks, &request_struct_name,  common_structures))$['\r']


            $['\n']
            #[derive(Debug, PartialEq)]$['\r']
            pub struct $(response_struct_name.clone()) {$['\r']
                $(ref toks => self.response.generate_members(toks))$['\r']
            }
            $['\n']
            $(ref toks => self.response.generate_deserializations(toks, response_struct_name.clone()))$['\r']
            $['\n']

            $(ref toks => self.generate_send(toks, &request_struct_name, &response_struct_name, common_structures))$['\r']

            //impl<SPI: SpiDevice> Transmit<SPI, $(command_name.to_case(Case::UpperCamel))Response> for $(command_name.to_case(Case::UpperCamel))Request {}
            impl<SPI: SpiDevice> Transmit<SPI, $(response_struct_name)> for $(request_struct_name.clone()) {}

            impl Command for $request_struct_name  {
                fn opcode(&self) -> u8 {
                    $(format!("0x{:X}", self.opcode))
                }
            }





        );

        output_file(file, tokens)?;

        Ok(())
    }

    //$(ref toks => self.generate_send(toks, &response_struct_name))$['\r']
    pub fn generate_send(
        &self,
        tokens: &mut Tokens<Rust>,
        request_name: &RequestStructName,
        response_name: &ResponseStructName,
        common_structures: &HashMap<String, CommonStructure>,
    ) {
        // let cased_request_name = request_name.to_case(Case::UpperCamel);
        // let cased_response_name = response_name.to_case(Case::UpperCamel);

        match &self.flow_control {
            FlowControl::Direct => {
                self.generate_direct_send(tokens, request_name, response_name, common_structures)
            }
            FlowControl::Polled { on, condition } => self.generate_polled_send(
                tokens,
                request_name,
                response_name,
                common_structures,
                on,
                condition,
            ),
        };

        // quote_in!(*tokens =>
        //     impl $(cased_request_name.clone()) {
        //         // This needs to be generated as we need to corrected specifiy the sizes
        //         // of the request and response.
        //         pub fn send<SPI: SpiDevice>(&self, spi: &mut SPI) -> Result<$(cased_response_name), DeviceError> {
        //             const REQUEST_BUF_LEN: usize = $(self.request.buffer_size(common_structures));
        //             const RESPONSE_BUF_LEN: usize = $(self.response.buffer_size(common_structures));

        //             let response = self.transmit::<REQUEST_BUF_LEN, RESPONSE_BUF_LEN>(spi)?;
        //             Ok(response)
        //         }
        //     }
        // );
    }

    // Generate the send funtion for a DIRECT flow control
    fn generate_direct_send(
        &self,
        tokens: &mut Tokens<Rust>,
        request_name: &RequestStructName,
        response_name: &ResponseStructName,
        common_structures: &HashMap<String, CommonStructure>,
    ) {
        quote_in!(*tokens =>
            impl $(request_name) {
                // This needs to be generated as we need to corrected specifiy the sizes
                // of the request and response.
                pub fn send<SPI: SpiDevice>(&self, spi: &mut SPI) -> Result<$(response_name), DeviceError> {
                    const REQUEST_BUF_LEN: usize = $(self.request.buffer_size(common_structures));
                    const RESPONSE_BUF_LEN: usize = $(self.response.buffer_size(common_structures));

                    let response = self.transmit::<REQUEST_BUF_LEN, RESPONSE_BUF_LEN>(spi)?;
                    Ok(response)
                }
            }
        )
    }

    // Generate the send function for a POLLED flow control.
    fn generate_polled_send(
        &self,
        tokens: &mut Tokens<Rust>,
        request_name: &RequestStructName,
        response_name: &ResponseStructName,
        common_structures: &HashMap<String, CommonStructure>,
        on: &str,
        condition: &str,
    ) {
        let header_structure = common_structures.get(on).unwrap(); //TODO Error handling
        let header_structure_buf_size = header_structure.buffer_size();
        let cased_header_structure_name = on.to_case(Case::UpperCamel); // TOO this should be replaced with naming module functions
        let request_buf_size = self.request.buffer_size(common_structures);
        let response_buf_size = self.response.buffer_size(common_structures);

        quote_in!(*tokens =>
            impl $request_name {
            pub fn send<SPI: SpiDevice>(&self, spi: &mut SPI) -> Result<$response_name, DeviceError> {
                let f = | h: $(cased_header_structure_name.clone())  | h.$condition;

                const REQUEST_BUF_LEN: usize = $request_buf_size;
                const RESPONSE_BUF_LEN: usize = $response_buf_size;
                const STATUS_HEADER_LEN: usize = $header_structure_buf_size;

                let response = self.polled_transmit::<REQUEST_BUF_LEN, RESPONSE_BUF_LEN, $cased_header_structure_name, STATUS_HEADER_LEN>(spi, f)?;

                Ok(response)
            }
        })
    }

    pub fn providers(&self) -> impl Iterator<Item = String> {
        let providers: Vec<String> = self
            .request
            .fields()
            .filter_map(|f| f.provider())
            .map(String::from)
            .collect();
        providers.into_iter()
    }
}
