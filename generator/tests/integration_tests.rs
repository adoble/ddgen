use std::fs::{self};
use std::path::{Path, PathBuf};
use std::str;

use pretty_assertions::assert_eq;
use tempfile::Builder;

mod utilities;

#[test]
#[ignore]
fn generate_simple() {
    let definition = include_str!("resources/simple.toml");
    let expected = include_str!("resources/test_command.rs");

    let temp_dir = Builder::new().prefix("ddgen_").tempdir().unwrap();

    let generated_dir: PathBuf = [temp_dir.path(), Path::new("generated")].iter().collect();
    fs::create_dir_all(&generated_dir).unwrap();

    generate::generate(&generated_dir, &None, false, &None, definition);

    // Read the generated command file and compare
    let gen_file_path: PathBuf = [
        temp_dir.path(),
        Path::new("generated/simple/src/test_command.rs"),
    ]
    .iter()
    .collect();
    let buf = fs::read(gen_file_path).unwrap();
    let actual = str::from_utf8(&buf).unwrap();

    // As formatting can change, strip all white space from the strings.
    let expected_clean = utilities::clean_spaces_tabs(expected);
    let actual_clean = utilities::clean_spaces_tabs(actual);

    assert_eq!(actual_clean, expected_clean);
}

#[test]
#[ignore]
fn generate_bits_fields_types() {
    let definition = include_str!("resources/bits_fields_types.toml");
    let expected = include_str!("resources/bft_test_command.rs");

    let temp_dir = Builder::new().prefix("ddgen_").tempdir().unwrap();

    let generated_dir: PathBuf = [temp_dir.path(), Path::new("generated")].iter().collect();
    fs::create_dir_all(&generated_dir).unwrap();

    generate::generate(&generated_dir, &None, false, &None, definition);

    // Read the generated command file and compare
    let gen_file_path: PathBuf = [
        temp_dir.path(),
        Path::new("generated/bits_fields_types/src/bft_test_command.rs"),
    ]
    .iter()
    .collect();
    let buf = fs::read(gen_file_path).unwrap();
    let actual = str::from_utf8(&buf).unwrap();

    // As formatting can change, strip all white space from the strings.
    let expected_clean = utilities::clean_spaces_tabs(expected);
    let actual_clean = utilities::clean_spaces_tabs(actual);

    assert_eq!(actual_clean, expected_clean);
}
