#![allow(dead_code)]
#![deny(unused_must_use)]

use std::{env, fs::OpenOptions, io::Write, path::PathBuf};

use cargo_toml::Manifest;
use crossterm::style::Stylize;

use xshell::cmd;

fn main() -> Result<(), anyhow::Error> {
    let args = env::args().skip(1).collect::<Vec<_>>();
    let args = args.iter().map(|s| &**s).collect::<Vec<_>>();

    match &args[..] {
        ["gen", _] => gen(args[1]),
        //["gen", "tests", _] => gen_with_tests(args[2]),

        // ["test", "host"] => test_host(),
        // ["test", "host-target"] => test_host_target(),
        // ["test", "target"] => test_target(),
        _ => {
            println!(
                "{}",
                "USAGE: cargo xtask gen [tests <test file name> ] <device name>".yellow()
            );
            Ok(())
        }
    }
}

fn gen(definition: &str) -> Result<(), anyhow::Error> {
    let _p = xshell::pushd(root_dir())?;
    println!("Generating for definition {}", definition);

    // Generate the driver for the specified definition file under <root>/generated/<device name>
    cmd!("cargo run --bin generator --   ./definitions/{definition}.toml ./generated").run()?;

    // Need to add the generated project to the workspace members
    // ASSUMING that the device name and the definition name are the same
    let member = format!("generated/{definition}");
    //add_workspace_member(&member)?;

    Ok(())
}

fn gen_with_tests(definition: &str) -> Result<(), anyhow::Error> {
    let _p = xshell::pushd(root_dir())?;
    println!("Generating for definition {definition} with tests");

    // The test code is ASSUMED to be under <project root>/device_tests/<name of device>/mod.rs
    // ASSUMING that root dir is correct so only using unwrap().
    let test_code_path: PathBuf = [
        root_dir().to_str().unwrap(),
        "device-tests",
        definition,
        "mod.rs",
    ]
    .iter()
    .collect();

    // Again using an unwrap as certain that this works.
    let test_code_path = test_code_path.to_str().unwrap();

    // Generate the driver for the specified definition file under <root>/generated/<device name>
    cmd!("cargo run --bin ddgen --  --tests {test_code_path} ./definitions/{definition}.toml ./generated")
        .run()?;

    // Need to add the generated project to the workspace members
    // ASSUMING that the device name and the definition name are the same
    let member = format!("generated/{definition}");
    add_workspace_member(&member)?;

    Ok(())
}

fn gen_spi() -> Result<(), anyhow::Error> {
    todo!()
}

fn root_dir() -> PathBuf {
    let mut xtask_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    xtask_dir.pop();

    xtask_dir
}

fn add_workspace_member(new_member: &str) -> Result<(), anyhow::Error> {
    let mut cargo_toml_path = root_dir();
    cargo_toml_path.push("Cargo.toml");

    let mut manifest = Manifest::from_path(&cargo_toml_path)?;
    let mut workspace = manifest
        .workspace
        .ok_or(XtaskErrors::WorkspaceNotFound(cargo_toml_path.clone()))?;

    // Add a workspace only if it is not already there. This stops the member
    // list in the workspace Cargo.toml containing repeats when the generator
    // is run more than once.
    if !workspace.members.contains(&new_member.to_string()) {
        workspace.members.push(new_member.to_string());
    }

    // Need to replace the whole workspace option
    manifest.workspace = Some(workspace);

    let toml = toml::to_string(&manifest).unwrap();

    let mut file = OpenOptions::new().write(true).open(cargo_toml_path)?;
    file.write_all(toml.as_bytes())?;

    Ok(())
}

#[derive(Debug)]
pub enum XtaskErrors {
    WorkspaceNotFound(PathBuf),
}

impl std::fmt::Display for XtaskErrors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::WorkspaceNotFound(path) => {
                write!(f, "No workspace path found in {}", path.display())
            }
        }
    }
}

impl std::error::Error for XtaskErrors {}
