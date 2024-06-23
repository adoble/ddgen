use genco::prelude::*;

use convert_case::{Case, Casing};
use std::fmt::Display;

#[derive(Clone)]
pub struct CommonStructureName(String);

impl CommonStructureName {
    pub fn to_file_name(&self) -> String {
        let name = self.0.to_case(Case::Snake);
        format!("{name}.rs")
    }
}

impl From<String> for CommonStructureName {
    fn from(name: String) -> Self {
        let formatted_name = name.to_case(Case::UpperCamel);
        CommonStructureName(formatted_name)
    }
}

impl From<&str> for CommonStructureName {
    fn from(name: &str) -> Self {
        CommonStructureName::from(name.to_string())
    }
}

impl Display for CommonStructureName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FormatInto<Rust> for CommonStructureName {
    fn format_into(self, tokens: &mut Tokens<Rust>) {
        quote_in!(*tokens => $(self.to_string()));
    }
}

impl FormatInto<Rust> for &CommonStructureName {
    fn format_into(self, tokens: &mut Tokens<Rust>) {
        quote_in!(*tokens => $(self.to_string()));
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_command_name_string() {
        let command_name = CommonStructureName::from("get_sys_state".to_string());

        let s: String = command_name.to_string();

        assert_eq!(s, "GetSysState".to_string());
    }

    #[test]
    fn test_command_name_str() {
        let command_name = CommonStructureName::from("get_sys_state");

        let s = command_name.to_string();

        assert_eq!(s, "GetSysState");
    }

    #[test]
    fn test_command_name_token() {
        let command_name = CommonStructureName::from("get_sys_state");
        let tokens: rust::Tokens = quote! {$(command_name)};

        assert_eq!("GetSysState\n", tokens.to_file_string().unwrap());
    }

    #[test]
    fn test_file_name() {
        let command_name = CommonStructureName::from("get_sys_state");
        assert_eq!("get_sys_state.rs", command_name.to_file_name());
    }
}
