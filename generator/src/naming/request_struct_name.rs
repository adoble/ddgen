use genco::prelude::*;
use std::fmt::Display;

use crate::naming::command_name::CommandName;

#[derive(Clone)]
pub struct RequestStructName(String);

impl From<&CommandName> for RequestStructName {
    fn from(command_name: &CommandName) -> Self {
        let name = format!("{}Request", command_name);
        RequestStructName(name)
    }
}

impl Display for RequestStructName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FormatInto<Rust> for RequestStructName {
    fn format_into(self, tokens: &mut Tokens<Rust>) {
        quote_in!(*tokens => $(self.to_string()));
    }
}

impl FormatInto<Rust> for &RequestStructName {
    fn format_into(self, tokens: &mut Tokens<Rust>) {
        quote_in!(*tokens => $(self.to_string()));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_name_creation() {
        let command_name = CommandName::from("get_sys_state");
        let request_name = RequestStructName::from(&command_name);

        let s: String = request_name.to_string();

        assert_eq!(s, "GetSysStateRequest".to_string());
    }

    #[test]
    fn test_request_name_token() {
        let command_name = CommandName::from("get_sys_state");
        let request_struct_name = RequestStructName::from(&command_name);
        let tokens: rust::Tokens = quote! {$(request_struct_name)};

        assert_eq!("GetSysStateRequest\n", tokens.to_file_string().unwrap());
    }
}
