use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use tempfile::{tempdir, Builder};

use xshell::{cmd, Shell};
#[ignore]
#[test]
fn generate_simple() {
    // Load the definition file for this test as resource
    let definition = include_bytes!("resources/simple.toml");

    let temp_dir = Builder::new().prefix("ddgen_").tempdir().unwrap();
    let definitions_dir: PathBuf = [temp_dir.path(), Path::new("definitions")].iter().collect();
    fs::create_dir_all(definitions_dir).unwrap();

    let def_file: PathBuf = [temp_dir.path(), Path::new("definitions/simple.toml")]
        .iter()
        .collect();
    fs::write(&def_file, definition).unwrap();

    let generated_dir: PathBuf = [temp_dir.path(), Path::new("generated")].iter().collect();
    fs::create_dir_all(&generated_dir).unwrap();

    let sh = Shell::new().unwrap();

    let _p = sh.change_dir(temp_dir.path());

    cmd!(sh, "ls").run().unwrap();

    // Generate the driver for the specified definition file under <root>/generated/<device name>
    // let gen_dir_name = generated_dir.as_os_str();
    // let def_file_name = def_file.as_os_str();
    // let gen_dir_name = generated_dir.to_str().unwrap();
    // let def_file_name = def_file.to_str().unwrap();

    let gen_dir_name = "./generated";
    let def_file_name = "./definitions/simple.toml";

    cmd!(
        sh,
        "cargo run --bin generator --   {def_file_name} {gen_dir_name}"
    )
    .run()
    .unwrap();
}
