use serde::Deserialize;

use std::{collections::HashMap, fs, path::Path};

use anyhow::Context;
use std::fs::File;
use std::io::Write;
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

    // Note: Using default here rather than Option as the default - an empty hash map -
    // make the logic easier.
    #[serde(rename = "struct", default)]
    //pub(crate) common_structures: Option<HashMap<String, CommonStructure>>,  See line 86
    pub(crate) common_structures: HashMap<String, CommonStructure>,

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
        println!(
            "{} in {}",
            "Generating code".bold(),
            out_path.as_os_str().to_str().unwrap()
        );

        let source_path_buf = self.generate_package_structure(out_path)?;
        let source_path = &source_path_buf.as_path();

        self.generate_common(source_path)?;

        // if let Some(common_structures) = &self.common_structures {
        //     self.generate_common_structure_files(source_path, common_structures)?;
        // }
        self.generate_common_structure_files(source_path, &self.common_structures)?;

        self.generate_commands(source_path, &self.common_structures)?;

        // let _tokens = rust::Tokens::new();

        // self.generate_register_block(source_path)?;
        // for register in self.registers.iter() {
        //     register.1.generate_register(register.0, source_path)?;
        // }

        self.generate_types_file(source_path)?;

        // if let Some(test_code_path) = tests_path {
        //     self.generate_tests(out_path, test_code_path)?;
        // }

        self.generate_lib(source_path, tests_path.is_some())?;

        Ok(())
    }

    fn generate_package_structure(&self, out_path: &Path) -> anyhow::Result<PathBuf> {
        let mut source_path_buf = PathBuf::from(out_path);
        source_path_buf.push(self.device.name.as_str().to_lowercase());
        source_path_buf.push("src");

        fs::create_dir_all(&source_path_buf)
            .with_context(|| "Unable to create output directory")?;

        let package_root = source_path_buf.parent().unwrap();

        // Create a Cargo.toml file
        let cargo_toml_str = cargo_gen::generate(self.device.name.as_str(), &self.version);
        let mut cargo_toml_path: PathBuf = package_root.to_path_buf();
        cargo_toml_path.push("Cargo.toml");

        let mut file = std::fs::File::create(cargo_toml_path).expect("create Cargo.toml failed");
        file.write_all(cargo_toml_str.as_bytes())
            .expect("write Cargo.toml failed");

        Ok(source_path_buf)
    }

    fn generate_lib(&self, out_path: &Path, _tests: bool) -> anyhow::Result<()> {
        println!("Generating lib");
        // Create a file for the lib file
        let lib_path: PathBuf = [out_path, Path::new("lib.rs")].iter().collect();
        let file = File::create(lib_path).with_context(|| "Cannot crate  lib file")?;

        let mut tokens = rust::Tokens::new();

        quote_in!(tokens =>
           #![cfg_attr(not(test), no_std)]

            //Reexports  TODO have this as a comment in the code


           pub use crate::types::*;
           pub use crate::error::DeviceError;


           $(for name in self.common_structures.keys() join(;$['\r'])=>  pub mod $(name.to_lowercase()) );

            // $(if self.enumerations.is_some() => $['\n']pub mod types;  )  TODO
           $(for name in self.commands.keys() join(;$['\r'])=>  pub mod $(name.to_lowercase()) );

           pub mod error;
           mod types;
           mod deserialize;
           mod serialize;
           mod request;
           mod response;
           mod bits;

        //     $(if tests => $['\n']  #[cfg(test)] $['\n'] mod tests;)

        );

        output_file(file, tokens)?;

        // // TODO update to include types/enumerations

        Ok(())
    }

    fn generate_commands(
        &self,
        out_path: &Path,
        common_structures: &HashMap<String, CommonStructure>,
    ) -> anyhow::Result<()> {
        for (command_name, command) in &self.commands {
            command.generate_command(command_name, common_structures, out_path)?;
        }

        Ok(())
    }

    /// Generates a file `types.rs` that contains any type defined in the device definition,
    /// especially the enumerations.
    /// Note: the type are generated for the whole devive rather than for individual registers as:
    /// * Imports are simpler.
    /// * They can be reused for different registers.
    fn generate_types_file(&self, out_path: &Path) -> anyhow::Result<()> {
        println!("Generating types");
        // Create a file for the types
        let types_path: PathBuf = [out_path, Path::new("types.rs")].iter().collect();

        let file = File::create(types_path).with_context(|| "Cannot open output file")?;

        let mut tokens = rust::Tokens::new();

        let doc_comment = DocComment::from_string("Types used in the driver");
        let generated_doc_comment = DocComment::from_string(&format!(
            "Generated with version {} of {}",
            VERSION, PKG_NAME
        ));

        quote_in!(tokens =>
                    $(doc_comment.as_string())$['\r']
                    $(DocComment::empty())$['\r']
                    $(generated_doc_comment.as_string())$['\r']
                    $['\n']
                    use crate::error::DeviceError;
                    $['\n']
        );

        if let Some(enumerations) = &self.enumerations {
            for enumeration in enumerations {
                let enum_identifier = &enumeration.0.to_case(Case::UpperCamel);

                quote_in!(tokens =>
                    #[derive(PartialEq, Debug, Copy, Clone)]
                    pub enum $enum_identifier {
                        $(ref toks {self.generate_enum_items(toks, enumeration.1)})
                    }
                    $['\n']

                    impl TryFrom<u8> for $enum_identifier {
                        type Error = DeviceError;
                        fn try_from(value: u8) -> Result<Self, Self::Error> {
                            match value {
                                $(ref toks {self.generate_enum_mappings(toks, enumeration.1)})
                                _ => Err(DeviceError::EnumConversion),
                            }
                        }
                    }
                    $['\n']
                );
            }
        };

        output_file(file, tokens)?;

        Ok(())
    }

    fn generate_enum_items(&self, tokens: &mut Tokens<Rust>, enumeration: &Enumeration) {
        // struct Enumeration(HashMap<String, u8>);

        for item in enumeration.0.iter() {
            let name = item.0.to_case(Case::UpperCamel);
            let descriminate = item.1.to_string();
            quote_in!(*tokens =>
                $name = $descriminate,
                $['\r']
            );
        }
    }

    fn generate_enum_mappings(&self, tokens: &mut Tokens<Rust>, enumeration: &Enumeration) {
        for item in enumeration.0.iter() {
            let name = item.0.to_case(Case::UpperCamel);
            let descriminate = item.1.to_string();
            quote_in!(*tokens =>
                $descriminate => Ok(Self::$name),
                $['\r']
            );
        }
    }

    fn generate_common_structure_files(
        &self,
        source_path: &Path,
        common_structures: &HashMap<String, CommonStructure>,
    ) -> anyhow::Result<()> {
        println!("Generate common structure files ...");
        for (name, structure) in common_structures {
            let name = name.to_case(Case::Lower);
            let file_name = format!("{name}.rs");
            let path: PathBuf = [source_path, Path::new(&file_name)].iter().collect();
            let file = File::create(path).with_context(|| format!("Cannot open {file_name}"))?;

            let mut tokens = rust::Tokens::new();

            let doc_comment = DocComment::from_string("Common structure used in the driver");
            let generated_doc_comment =
                DocComment::from_string(&format!("Generated with version {} of ddgen", VERSION));

            quote_in!(tokens =>
                $(doc_comment.as_string())$['\r']
                $(DocComment::empty())$['\r']
                $(generated_doc_comment.as_string())$['\r']

                use crate::deserialize::Deserialize;
                use crate::error::DeviceError;
                use crate::request::{RequestArray, RequestBit, RequestField, RequestWord};
                use crate::response::{ResponseArray, ResponseBit, ResponseField, ResponseWord};
                use crate::serialize::Serialize;

                use crate::types::*;

                $(ref toks {structure.generate(toks, name)})

                $(ref toks {structure.generate_serializations(toks, name)})

                $(ref toks {structure.generate_deserializations(toks, name)})

            );

            output_file(file, tokens)?;
        }

        Ok(())
    }

    fn generate_common(&self, out_path: &Path) -> anyhow::Result<()> {
        println!("Transferring common code over.");

        // Need to use include_str!() so as to bind the common files to the binary
        // as resources. However, include_str!() only accepts str literals so have to
        // initialise the hashmap in this akward way.
        let common_resources = HashMap::from([
            ("bits.rs", include_str!("../../common/src/bits.rs")),
            (
                "deserialize.rs",
                include_str!("../../common/src/deserialize.rs"),
            ),
            ("error.rs", include_str!("../../common/src/error.rs")),
            ("request.rs", include_str!("../../common/src/request.rs")),
            ("response.rs", include_str!("../../common/src/response.rs")),
            (
                "serialize.rs",
                include_str!("../../common/src/serialize.rs"),
            ),
        ]);

        for (file_name, code_resource) in &common_resources {
            let lib_path: PathBuf = [out_path, Path::new(file_name)].iter().collect();
            let mut file = File::create(lib_path).with_context(|| "Cannot open output file")?; //TODO make the context useful
            file.write_all(code_resource.as_bytes())?;
        }

        Ok(())
    }

    fn generate_tests(&self, _out_path: &Path, _test_code_path: &Path) -> anyhow::Result<()> {
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
