#![allow(dead_code)]

// TODO:
// - Add some more integration tests
// - Common structure serialisation
// - Common structure file names and struct name could confiict with common file names.
//   Move them to a seperate module to avoid this.
// - The bit_spec definition for a common structure can lead to conflicts  with how the specification writer
//   defines the bit_spec for a command. For instance:
//       [commands.TUNE.request]
//       a_header = {bits = "0[]", type = "header"}
//       [struct.header]
//       status = {bits = "0[]"}
//       extra_status = {bits = "1[]"}
//   Need to think about this. Consider that the position where a struct is placed in the bit stream
//   has to be specified. Maybe need a position  attribute, e.g. a_header = {struct = "header", position = 0}?.
//   And also checks that enough space is left.
//  - Replace members HashMap with (the crate) BiMap. This simplifys the code by removing the need to
//    construct the symbol table in Members.
// - Run clippy
// - Incorrect bit_specs seem to be processed without giving an error leading to incorrectly generated code
//   ( e.g. incorrect syntax with bit fields)
// - The generated Cargo.toml file needs to have means to update the depedendency version numbers.
// - Often need to pass HashMap<String, Field> into functions as this forms the symbol table.
//   Could change this to a new type (e.g. SymbolTable(<HashMap<String, Field>)), but need to
//   see if this can be done easily with serde (see https://github.com/softprops/dynomite/pull/145).
//  - Merge field::TargetType with bit_spec::BitSpecType
//  - Move common/src/test/.. into common/tests (as was dome for generator)
//  - Ensure that all types are handled in serailize and deserialize
//  - Need to handle big endian encodings
//  - Replace the comment generation with something like this: https://github.com/udoprog/genco/issues/53#issuecomment-1821318498
//  - Move the type Enumeration out of lib.rs and into a new file. Move over any functions that handle enumeration.
//  - Is access.rs required?
// use serde::Deserialize;
use std::{fs::File, path::PathBuf};

use clap::Parser;
//use crossterm::style::Stylize;
// use std::collections::HashMap;
use std::io::Read;

//use crate::{definition::Definition, error_reporting::error_report};
//use generate;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// The path to the file to read
    in_path: std::path::PathBuf,

    /// The path where the project is generated.
    /// The project name if the device name in the
    /// in the definitions file.
    out_path: std::path::PathBuf,

    /// Name of the generated project.
    /// Overrides the device name in the definitions file.
    #[arg(short, long)]
    name: Option<String>,

    /// Path to test code.
    /// This will be copied into the generated project structure
    #[arg(long, short)]
    tests: Option<PathBuf>,

    /// Explicity exclude the generated code from any workspaces.
    #[arg(long)]
    ws_exclude: bool,
}

fn main() {
    let args = Cli::parse();

    let definition_file_name = String::from(args.in_path.to_string_lossy());

    let mut file = match File::open(args.in_path) {
        Ok(file) => file,
        Err(_error) => {
            println!("Unable to open definition file {}", definition_file_name);
            return;
        }
    };

    let mut toml_specification = String::new();

    println!("Reading definition");

    file.read_to_string(&mut toml_specification).unwrap();

    generate::generate(&args.out_path, &args.tests, toml_specification.as_str());
}
