#![allow(dead_code)]

// TODO:
// - Run clippy
// - Change serialize so that the const generics are in the function
//   and not in the trait.
// - The generated Cargo.toml file needs to have means to update the depedendency version numbers.
// - Need to format the code after generation. (Can we run cargo fmt at the end?)
// - Often need to passs HashMap<String, Field> into functions as this forms the symbol table.
//   Could change this to a new type (e.g. SymbolTable(<HashMap<String, Field>)), but need to
//   see if this can be done easily with serde (see https://github.com/softprops/dynomite/pull/145).
//  - Merge field::TargetType with bit_spec::BitSpecType
//  - Ensure that all types are handled in serailize and deserialize
//  - Need to handle big endian encodings
use serde::Deserialize;
use std::{fs::File, path::PathBuf};

use clap::Parser;
use crossterm::style::Stylize;
use std::{collections::HashMap, io::Read};

use crate::{definition::Definition, error_reporting::error_report};

mod access;
//mod bit_range;
mod cargo_gen;
mod command;
mod common_structure;
mod definition;
mod doc_comment;
mod error_reporting;
mod field;
mod output;

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

// TODO move this to another file as for the other serde structs.
#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
struct Enumeration(HashMap<String, u8>);

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

    //let mut file = File::open(args.in_path).unwrap();

    let mut contents = String::new();

    println!("Reading definition");

    file.read_to_string(&mut contents).unwrap();

    let parse_result: Result<Definition, toml::de::Error> = toml::from_str(contents.as_str());
    match parse_result {
        Ok(definition) => {
            definition
                .generate_code(&args.out_path, &args.tests)
                .expect("Unable to generate driver code");
            println!("{}", "Finished generation!".green());
        }
        Err(err) => {
            error_report(contents.as_str(), err.message(), err.span())
            // let span = err.span();
            // println!("{} {:?}", err.message().red(), span)
        }
    }
}
