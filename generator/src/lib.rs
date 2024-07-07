// Seperated out for the generation integration tests.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crossterm::style::Stylize;
use serde::Deserialize;

use crate::{definition::Definition, error_reporting::error_report};

mod access;
mod cargo_gen;
mod command;
mod common_structure;
mod definition;
mod doc_comment;
mod error_reporting;
mod field;
mod flow_control;
mod members;
mod naming;
mod output;
mod providers;

// TODO move this to another file as for the other serde structs.
#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
struct Enumeration(HashMap<String, u8>);

/// Generate the code
pub fn generate(
    out_path: &Path,
    project_name: &Option<String>,
    gen_providers: bool,
    tests_path: &Option<PathBuf>,
    toml_specification: &str,
) {
    let parse_result: Result<Definition, toml::de::Error> = toml::from_str(toml_specification);

    match parse_result {
        Ok(definition) => {
            definition
                .generate_code(out_path, project_name, gen_providers, tests_path)
                .expect("Unable to generate driver code");
            println!("{}", "Finished generation!".green());
        }
        Err(err) => {
            // error_report(toml_specification.as_str(), err.message(), err.span())
            error_report(toml_specification, err.message(), err.span())
            // let span = err.span();
            // println!("{} {:?}", err.message().red(), span)
        }
    };
}
