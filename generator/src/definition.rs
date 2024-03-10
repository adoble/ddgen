use serde::Deserialize;

use std::{collections::HashMap, fs, path::Path};

use anyhow::Context;
use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;

use convert_case::{Case, Casing};
use crossterm::style::Stylize;
use genco::prelude::*;

use crate::cargo_gen;
use crate::command::Command;
use crate::common_structure::CommonStructure;
use crate::doc_comment::DocComment;
use crate::output::output_file;
use crate::Enumeration;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const PKG_NAME: &str = env!("CARGO_PKG_NAME");

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Definition {
    pub(crate) version: semver::Version,
    pub(crate) device: Device,

    // Note: Using the alias "commands" (in plural) seems to be a reserved name
    // in toml, at least when using vscode.
    #[serde[alias = "command"]]
    //pub(crate) registers: HashMap<String, Register>,
    pub(crate) commands: HashMap<String, Command>,

    #[serde(rename = "struct")]
    pub(crate) common_structures: Option<HashMap<String, CommonStructure>>,

    #[serde(rename = "enum")]
    pub(crate) enumerations: Option<HashMap<String, Enumeration>>,
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub enum Endian {
    #[serde(alias = "le")]
    #[serde(alias = "little_endian")]
    #[serde(alias = "little")]
    Little,
    #[serde(alias = "be")]
    #[serde(alias = "big_endian")]
    #[serde(alias = "big")]
    Big,
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Device {
    pub(crate) name: String,

    /// Word size
    pub(crate) word_size: u8,

    /// Endian of words used transmitted/received by the devive
    pub(crate) endian: Endian,
}

impl Definition {
    pub fn generate_code(
        &self,
        out_path: &Path,
        tests_path: &Option<PathBuf>,
    ) -> anyhow::Result<()> {
        // println!("{}", "Generating code ...".bold());

        // let source_path_buf = self.generate_package_structure(out_path)?;
        // let source_path = &source_path_buf.as_path();

        // let _tokens = rust::Tokens::new();

        // self.generate_register_block(source_path)?;
        // for register in self.registers.iter() {
        //     register.1.generate_register(register.0, source_path)?;
        // }

        // self.generate_types_file(source_path)?;

        // self.generate_common(source_path)?;

        // self.generate_errors(source_path)?;

        // if let Some(test_code_path) = tests_path {
        //     self.generate_tests(out_path, test_code_path)?;
        // }

        // self.generate_lib(source_path, tests_path.is_some())?;

        Ok(())
    }

    fn generate_package_structure(&self, out_path: &Path) -> anyhow::Result<PathBuf> {
        // let mut source_path_buf = PathBuf::from(out_path);
        // source_path_buf.push(self.device.name.as_str().to_lowercase());
        // source_path_buf.push("src");

        // fs::create_dir_all(&source_path_buf)
        //     .with_context(|| "Unable to create output directory")?;

        // let package_root = source_path_buf.parent().unwrap();

        // // Create a Cargo.toml file
        // let cargo_toml_str = cargo_gen::generate(self.device.name.as_str(), &self.version);
        // let mut cargo_toml_path: PathBuf = package_root.to_path_buf();
        // cargo_toml_path.push("Cargo.toml");

        // let mut file = std::fs::File::create(cargo_toml_path).expect("create Cargo.toml failed");
        // file.write_all(cargo_toml_str.as_bytes())
        //     .expect("write Cargo.toml failed");

        // Ok(source_path_buf)
        todo!()
    }

    fn generate_lib(&self, out_path: &Path, tests: bool) -> anyhow::Result<()> {
        // println!("Generating lib");
        // // Create a file for the lib file
        // let lib_path: PathBuf = [out_path, Path::new("lib.rs")].iter().collect();
        // let file = File::create(lib_path).with_context(|| "Cannot open output file")?;

        // let mut tokens = rust::Tokens::new();

        // quote_in!(tokens =>
        //     #![cfg_attr(not(test), no_std)]

        //     // Rexports
        //     pub use crate::register::Register;
        //     pub use crate::readable::Readable;
        //     pub use crate::writable::Writable;
        //     pub use crate::error::DeviceError;
        //     pub use crate::register_block::RegisterBlock;

        //     pub mod register_block;

        //     $(for name in self.registers.keys() join(;$['\r'])=>  pub mod $(name.to_lowercase()) );

        //     $(if self.enumerations.is_some() => $['\n']pub mod types;  )

        //     pub mod error;

        //     mod readable;
        //     mod writable;
        //     mod register;

        //     $(if tests => $['\n']  #[cfg(test)] $['\n'] mod tests;)

        // );

        // output_file(file, tokens)?;

        // // TODO update to include types/enumerations

        // Ok(())
        todo!()
    }

    fn generate_register_block(&self, out_path: &Path) -> anyhow::Result<()> {
        // println!("Generating register_block.rs");

        // // Create a file for the register block
        // let register_block_path: PathBuf =
        //     [out_path, Path::new("register_block.rs")].iter().collect();

        // let file = File::create(register_block_path).with_context(|| "Cannot open output file")?;

        // let mut tokens = rust::Tokens::new();

        // let reg_variables: Vec<String> = self
        //     .registers
        //     .keys()
        //     .map(|name| {
        //         format!(
        //             "{}: crate::{}::{}",
        //             name.to_lowercase(),
        //             name.to_lowercase(),
        //             name.to_uppercase()
        //         )
        //     })
        //     .collect();

        // quote_in!(tokens =>
        //   const BASE_ADDRESS: u8 = $(self.device.base_address);

        //   pub struct RegisterBlock {
        //       pub address: u8,

        //       // Registers
        //       $(for var in reg_variables.iter() join($['\r'])=> pub $var,)
        //   }

        //   impl RegisterBlock {
        //       pub fn new(offset_address: u8) -> Self {
        //           // TODO need a check on offset address
        //           Self {
        //               address: BASE_ADDRESS | offset_address,
        //               // Registers
        //               $(for name in reg_variables join(,$['\r'])=>  $name::new(offset_address))

        //           }
        //       }
        //   }

        // );

        // output_file(file, tokens)?;

        // Ok(())
        todo!()
    }

    /// Generates a file `types.rs` that contains any type defined in the device definition,
    /// especially the enumerations.
    /// Note: the type are generated for the whole devive rather than for individual registers as:
    /// * Imports are simpler.
    /// * They can be reused for different registers.
    fn generate_types_file(&self, out_path: &Path) -> anyhow::Result<()> {
        // println!("Generating types");
        // // Create a file for the types
        // let types_path: PathBuf = [out_path, Path::new("types.rs")].iter().collect();

        // let file = File::create(types_path).with_context(|| "Cannot open output file")?;

        // let mut tokens = rust::Tokens::new();

        // let doc_comment = DocComment::from_string("Types used in the driver");
        // let generated_doc_comment = DocComment::from_string(&format!(
        //     "Generated with version {} of {}",
        //     VERSION, PKG_NAME
        // ));

        // quote_in!(tokens =>
        //             $(doc_comment.as_string())
        //             $(DocComment::empty())
        //             $(generated_doc_comment.as_string())
        //             $['\n']
        //             use crate::error::DeviceError;
        //             $['\n']
        // );

        // if let Some(enumerations) = &self.enumerations {
        //     for enumeration in enumerations {
        //         let enum_identifier = &enumeration.0.to_case(Case::UpperCamel);

        //         quote_in!(tokens =>
        //             pub enum $enum_identifier {
        //                 $(ref toks {self.generate_enum_items(toks, enumeration.1)})
        //             }
        //             $['\n']

        //             impl TryFrom<u8> for $enum_identifier {
        //                 type Error = DeviceError;
        //                 fn try_from(value: u8) -> Result<Self, Self::Error> {
        //                     match value {
        //                         $(ref toks {self.generate_enum_mappings(toks, enumeration.1)})
        //                         _ => Err(DeviceError::EnumConversion),
        //                     }
        //                 }
        //             }
        //             $['\n']
        //         );
        //     }
        // };

        // output_file(file, tokens)?;

        // Ok(())
        todo!()
    }

    fn generate_enum_items(&self, tokens: &mut Tokens<Rust>, enumeration: &Enumeration) {
        // // struct Enumeration(HashMap<String, u8>);

        // for item in enumeration.0.iter() {
        //     let name = item.0.to_case(Case::UpperCamel);
        //     let descriminate = item.1.to_string();
        //     quote_in!(*tokens =>
        //         $name = $descriminate,
        //         $['\r']
        //     );
        // }
        todo!()
    }

    fn generate_enum_mappings(&self, tokens: &mut Tokens<Rust>, enumeration: &Enumeration) {
        // for item in enumeration.0.iter() {
        //     let name = item.0.to_case(Case::UpperCamel);
        //     let descriminate = item.1.to_string();
        //     quote_in!(*tokens =>
        //         $descriminate => Ok(Self::$name),
        //         $['\r']
        //     );
        // }
        todo!()
    }

    fn generate_common(&self, out_path: &Path) -> anyhow::Result<()> {
        // println!("Transferring common code over.");

        // let lib_path: PathBuf = [out_path, Path::new("register.rs")].iter().collect();
        // let mut file = File::create(lib_path).with_context(|| "Cannot open output file")?; //TODO make the context useful

        // let code_str = include_str!("../common/register_i2c.rs");

        // file.write_all(code_str.as_bytes())?;

        // let lib_path: PathBuf = [out_path, Path::new("writable.rs")].iter().collect();
        // let mut file = File::create(lib_path).with_context(|| "Cannot open output file")?; //TODO make the context useful
        // let code_str = include_str!("../common/writable.rs");
        // file.write_all(code_str.as_bytes())?;

        // let lib_path: PathBuf = [out_path, Path::new("readable.rs")].iter().collect();
        // let mut file = File::create(lib_path).with_context(|| "Cannot open output file")?; //TODO make the context useful
        // let code_str = include_str!("../common/readable.rs");
        // file.write_all(code_str.as_bytes())?;

        //Ok(())
        todo!()
    }

    fn generate_errors(&self, out_path: &Path) -> anyhow::Result<()> {
        // println!("Transferring error code block over");

        // let lib_path: PathBuf = [out_path, Path::new("error.rs")].iter().collect();
        // let mut file = File::create(lib_path).with_context(|| "Cannot open output file")?; //TODO make the context useful

        // let code_str = include_str!("../common/error.rs");

        // file.write_all(code_str.as_bytes())?;

        // Ok(())
        todo!()
    }

    fn generate_tests(&self, out_path: &Path, test_code_path: &Path) -> anyhow::Result<()> {
        // println!("{}", "Transferring test code over".yellow());
        // // Test code files are small so just reading into a string
        // let mut file =
        //     std::fs::File::open(test_code_path).with_context(|| "Cannot open test code file")?;

        // let mut contents = String::new();
        // file.read_to_string(&mut contents).unwrap();

        // // Test directory is is in the form
        // // <out path>/<device name = project name>/src/tests
        // let target_test_code_dir: PathBuf = [
        //     out_path,
        //     Path::new(&self.device.name.as_str().to_lowercase()),
        //     Path::new("src/tests"),
        // ]
        // .iter()
        // .collect();

        // // Do not need to check for previous existence as create_dir_all will
        // // not complain if any of the directories already exist.
        // fs::create_dir_all(&target_test_code_dir)
        //     .with_context(|| "Cannot create test directory")?;

        // // let target_test_code_path: PathBuf =
        // // [out_path, Path::new("stc/tests/mod.rs")].iter().collect();
        // let target_test_code_path: PathBuf = [target_test_code_dir, PathBuf::from("mod.rs")]
        //     .iter()
        //     .collect();

        // let mut target_test_code_file = File::create(target_test_code_path)
        //     .with_context(|| "Cannot create test output file")?; //TODO make the context useful

        // target_test_code_file.write_all(contents.as_bytes())?;

        // Ok(())
        todo!()
    }

    fn default_version() -> String {
        "0.0.0".to_string()
    }

    // fn output_file(&self, file: File, tokens: Tokens<Rust>) -> Result<(), anyhow::Error> {
    //     let mut writer = fmt::IoWriter::new(file);
    //     let fmt = fmt::Config::from_lang::<Rust>().with_indentation(fmt::Indentation::Space(2));
    //     let config = rust::Config::default();
    //     let format = rust::Format::default();
    //     tokens.format(&mut writer.as_formatter(&fmt), &config, &format)?;
    //     Ok(())
    // }
}
