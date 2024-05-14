#![allow(dead_code)]

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

    /// Specify if skeleton provider structs should be generated.
    /// Warning: This overwrites any previous modifications.
    #[clap(long, short, action)]
    providers: bool,

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

    generate::generate(
        &args.out_path,
        args.providers,
        &args.tests,
        toml_specification.as_str(),
    );
}
