use anyhow::Context;
use convert_case::{Case, Casing};
use genco::prelude::*;
use std::fs::File;
use std::path::{Path, PathBuf};

use crate::doc_comment::DocComment;
use crate::output::output_file;

const VERSION: &str = env!("CARGO_PKG_VERSION");

use crate::definition::Definition;

#[derive(Debug, Default)]
pub(crate) struct Providers(Vec<String>);

impl Providers {
    pub(crate) fn from_definition(definition: &Definition) -> Providers {
        let mut providers: Vec<String> = Vec::new();
        for command in definition.commands.iter() {
            providers.extend(command.1.providers());
        }

        Providers(providers)
    }

    pub(crate) fn provider_names(&self) -> impl Iterator<Item = &str> {
        self.0.iter().map(|s| s.as_str())
    }

    pub(crate) fn len(&self) -> usize {
        self.0.len()
    }

    pub(crate) fn generate(&self, out_path: &Path) -> anyhow::Result<()> {
        println!("Generating providers ...");

        for provider in self.0.iter() {
            // Create a file for the provider  file
            let provider_file_name = format!("{}.rs", provider.to_case(Case::Snake));
            let provider_path: PathBuf = out_path.join(provider_file_name);
            let provider_file =
                File::create(provider_path).with_context(|| "Cannot create  lib file")?;

            let description_doc_comment = DocComment::from_string("Provider iterator").as_string();
            let generated_doc_comment =
                DocComment::from_string(&format!("Generated with version {} of ddgen", VERSION))
                    .as_string();

            let tokens: rust::Tokens = quote!(
                $(description_doc_comment)
                $(generated_doc_comment)

                #[derive(Debug, PartialEq, Copy, Clone)]
                pub struct $(provider.to_case(Case::UpperCamel)) {}

                impl Iterator for $(provider.to_case(Case::UpperCamel)) {
                    type Item = u8;

                    fn next(&mut self) -> Option<Self::Item> {
                        todo!()
                    }
                }
            );
            output_file(provider_file, tokens)?;
        }

        Ok(())
    }
}
