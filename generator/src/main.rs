#![allow(dead_code)]

// TODO:
// - Add some more integration tests
// - Common structure serialisation
// - Run clippy
// - The generated Cargo.toml file needs to have means to update the depedendency version numbers.
// - Often need to passs HashMap<String, Field> into functions as this forms the symbol table.
//   Could change this to a new type (e.g. SymbolTable(<HashMap<String, Field>)), but need to
//   see if this can be done easily with serde (see https://github.com/softprops/dynomite/pull/145).
//  - Merge field::TargetType with bit_spec::BitSpecType
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
