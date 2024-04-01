use std::collections::HashMap;
use std::fs::File;
use std::path::{Path, PathBuf};

use anyhow::Context;
use convert_case::{Case, Casing};
use genco::prelude::*;
use serde::Deserialize;

use crate::access::Access;
use crate::doc_comment::DocComment;
use crate::field::Field;
use crate::output::output_file;

//use crate::generate::output_file; //TODO need to place this in it's own module

const VERSION: &str = env!("CARGO_PKG_VERSION");
const PKG_NAME: &str = env!("CARGO_PKG_NAME");

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Command {
    opcode: u8,

    description: Option<String>,

    request: HashMap<String, Field>,
    response: HashMap<String, Field>,
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
            $(command_doc_comment)$['\r']


            $(description_doc_comment)$['\r']


            $(generated_doc_comment)

            $['\n']
            #[derive(Debug, PartialEq, Eq)]$['\r']
            pub struct $(&request_struct_name) {$['\r']
                $(ref toks => self.generate_members(toks, &self.request))$['\r']
            }
            $['\n']
            $(ref toks => self.generate_serializations(toks, &request_struct_name, &self.request))$['\r']

            $['\n']
            #[derive(Debug, PartialEq, Eq)]$['\r']
            pub struct $(&response_struct_name) {$['\r']
                $(ref toks => self.generate_members(toks,  &self.request))$['\r']
            }
            $['\n']
            $(ref toks => self.generate_deserializations(toks, &response_struct_name,&self.request))$['\r']



        );

        output_file(file, tokens)?;

        Ok(())
    }

    fn generate_members(&self, tokens: &mut Tokens<Rust>, members: &HashMap<String, Field>) {
        for (name, field) in members {
            field.generate_struct_member(tokens, name);
            // quote_in!(*tokens =>
            //     pub $name : u8, $['\r']//$(field.type_as_str()),
            // );
        }
    }

    fn generate_serializations(
        &self,
        tokens: &mut Tokens<Rust>,
        struct_name: &str,
        members: &HashMap<String, Field>,
    ) {
        // let mut serialization_buffer_size = 0;
        // for f in members.values().filter(|&f| f.is_bitfield()) {
        //     if let Field::BitField { bit_range, .. } = f {
        //         println!("{:?} maxsize = {}", bit_range, bit_range.max_size());
        //         serialization_buffer_size += bit_range.max_size();
        //     }
        // }
        let mut serialization_buffer_size = self.buffer_size(members);

        // for f in members.values().filter(|&f| f.is_structure()) {
        //     if let Field::Structure { .. } = f {
        //         todo!();
        //     }
        // }

        quote_in!(*tokens =>
            impl Serialize for $(struct_name) {
                fn serialize<const N: usize>(&self) -> (u8, [u8; N]) {
                let mut data = [0u8, $(serialization_buffer_size)];

                $(for (name, field) in members => $(ref toks {field.generate_field_serialization(toks, field, name)}) )
                }
            }
        );
    }

    fn generate_deserializations(
        &self,
        tokens: &mut Tokens<Rust>,
        struct_name: &str,
        members: &HashMap<String, Field>,
    ) {
        for (name, field) in members {
            field.generate_field_deserialization(tokens, field, name);
        }
    }

    pub(crate) fn generate_register_preamble(&self, tokens: &mut Tokens<Rust>, name: &str) {
        // let register_address_hex = format!("{:#02x}", self.address);

        // let register_doc_comment = DocComment::from_string(&format!("Register {}", name));
        // let description_doc_comment =
        //     DocComment::from_string(self.description.as_ref().unwrap_or(&String::new()));
        // let generated_doc_comment = DocComment::from_string(&format!(
        //     "Generated with version {} of {}",
        //     VERSION, PKG_NAME
        // ));

        // quote_in!(*tokens =>
        //     $(register_doc_comment.as_string())
        //     $(for line in description_doc_comment.lines() => $line$['\r'])
        //     $(DocComment::empty())
        //     $(generated_doc_comment.as_string())
        //     $['\n']
        //     use embedded_hal::blocking::i2c::{Read, Write, WriteRead};

        //     use crate::register::Register;
        //     use crate::readable::Readable;
        //     use crate::writable::Writable;

        //     #[allow(unused_imports)]
        //     use crate::types::*;

        //     const REGISTER_ADDRESS: u8 = $register_address_hex;

        // );
        todo!();
    }

    pub(crate) fn generate_register_struct(&self, tokens: &mut Tokens<Rust>, name: &str) {
        // quote_in!(*tokens =>
        //     $['\n']
        //     pub struct $(name.to_uppercase()) {
        //         i2c_address: u8,
        //     }

        //     impl $(name.to_uppercase()) {
        //         pub fn new(i2c_address: u8) -> $(name.to_uppercase()) {
        //             $(name.to_uppercase()) { i2c_address }
        //         }
        //     }

        //     impl<I2C> Register<I2C, R, W> for $(name.to_uppercase())
        //     where
        //         I2C: Read + Write + WriteRead,
        //     {
        //         fn register_address(&self) -> u8 {
        //             REGISTER_ADDRESS
        //         }

        //         fn device_address(&self) -> u8 {
        //             self.i2c_address
        //         }

        //         fn reset_value(&self) -> u8 {
        //             $(match self.reset {
        //                 Some(value) => $(format!("{:#04x}", value)),
        //                 None => 0x00,
        //             } )

        //         }
        //     }

        // )
        todo!();
    }

    pub fn generate_register_reader(&self, tokens: &mut Tokens<Rust>) {
        // let is_readable = matches!(self.access, Access::Read | Access::ReadWrite);

        // let header_doc_comment = DocComment::from_string("Reader");

        // quote_in!(*tokens =>
        //     $['\n']
        //     $(header_doc_comment.as_string())
        //     $(if is_readable {pub$[' ']} )struct R {
        //         bits: u8,
        //     }

        //     impl Readable for R {
        //         fn new(bits: u8) -> Self {
        //             R { bits }
        //         }

        //         fn bits(&self) -> u8 {
        //             self.bits
        //         }
        //     }
        // );

        // if is_readable {
        //     self.generate_reader_fields(tokens)
        // };
        todo!();
    }

    pub fn generate_reader_fields(&self, tokens: &mut Tokens<Rust>) {
        // let fields = self.fields.as_ref();

        // if let Some(fields) = fields {
        //     quote_in!(*tokens =>
        //         $['\n']
        //         impl R {
        //             $(for f in fields.iter() => $(ref toks {f.1.generate_read_field(toks, f.0)}) )
        //         }
        //     );
        // }
        todo!();
    }

    pub fn generate_register_writer(&self, tokens: &mut Tokens<Rust>) {
        // let is_writable = matches!(self.access, Access::Write | Access::ReadWrite);

        // let header_doc_comment = DocComment::from_string("Writer");

        // quote_in!(*tokens =>
        //     $['\n']
        //     $(header_doc_comment.as_string())
        //     $(if is_writable {pub$[' ']})struct W {
        //         bits: u8,
        //     }
        //     impl Writable for W {
        //         fn new(bits: u8) -> Self {
        //             W { bits }
        //         }

        //         fn bits(&self) -> u8 {
        //             self.bits
        //         }

        //         fn set_bits(&mut self, bits: u8) -> &mut Self {
        //             self.bits = bits;
        //             self
        //         }
        //     }
        // );

        // if is_writable {
        //     self.generate_writer_fields(tokens)
        // };
        todo!();
    }

    pub fn generate_writer_fields(&self, tokens: &mut Tokens<Rust>) {
        // let fields = self.fields.as_ref();

        // if let Some(fields) = fields {
        //     quote_in!(*tokens =>
        //         $['\n']
        //         impl W {
        //             $(for f in fields.iter() => $(ref toks {f.1.generate_write_field(toks, f.0)}) )
        //         }
        //     );
        // }
        todo!();
    }

    pub fn generate_enums(tokens: &mut Tokens<Rust>, _name: &str) {
        // quote_in!(*tokens =>
        //     $['\n']
        //     /// Enumerations
        //     todo!()

        // );
        todo!();
    }

    pub fn buffer_size(&self, members: &HashMap<String, Field>) -> usize {
        let mut buffer_size = 0;
        let mut positions: HashMap<usize, usize> = HashMap::new();
        for f in members.values() {
            // Need to implement this with a hashmap as more than one bit spec can reference the
            // same position in the buffer. The key is the position and the value is the size.
            match f {
                Field::BitField { bit_range, .. } => {
                    if !positions.contains_key(&bit_range.start.index) {
                        positions.insert(bit_range.start.index, bit_range.max_size());
                    }
                    //buffer_size += bit_range.max_size()
                }
                Field::Structure {
                    common_structure_name,
                } => todo!(),
            }
        }

        positions.values().fold(0, |acc, size| acc + size)
    }
}
