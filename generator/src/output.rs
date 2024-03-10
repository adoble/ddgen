use std::fs::File;

use genco::{fmt, prelude::*};

pub fn output_file(file: File, tokens: Tokens<Rust>) -> Result<(), anyhow::Error> {
    let mut writer = fmt::IoWriter::new(file);
    let fmt = fmt::Config::from_lang::<Rust>().with_indentation(fmt::Indentation::Space(2));
    let config = rust::Config::default();
    let format = rust::Format::default();
    tokens.format(&mut writer.as_formatter(&fmt), &config, &format)?;
    Ok(())
}
