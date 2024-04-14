use std::fs::{self};
use std::path::{Path, PathBuf};
use std::str;

use tempfile::Builder;

pub fn clean_spaces_tabs(input: &str) -> String {
    let mut result = String::new();
    let mut last_char: Option<char> = None;

    for c in input.chars() {
        match c {
            ' ' | '\t' | '\n' => {
                if last_char != Some(' ') {
                    result.push(' ');
                }
                last_char = Some(' ');
            }
            '\r' => (),
            _ => {
                result.push(c);
                last_char = Some(c);
            }
        }
    }

    result
}
