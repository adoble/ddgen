use crate::naming::command_name::CommandName;
use genco::prelude::*;
use std::fmt::Display;

#[derive(Clone)]
pub struct ResponseStructName(String);

impl From<&CommandName> for ResponseStructName {
    fn from(command_name: &CommandName) -> Self {
        let name = format!("{}Response", command_name.to_string());
        ResponseStructName(name)
    }
}

impl Display for ResponseStructName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FormatInto<Rust> for ResponseStructName {
    fn format_into(self, tokens: &mut Tokens<Rust>) {
        quote_in!(*tokens => $(self.to_string()));
    }
}

impl FormatInto<Rust> for &ResponseStructName {
    fn format_into(self, tokens: &mut Tokens<Rust>) {
        quote_in!(*tokens => $(self.to_string()));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_response_struct_name_creation() {
        let command_name = CommandName::from("get_sys_state");
        let response_struct_name = ResponseStructName::from(&command_name);

        let s: String = response_struct_name.to_string();

        assert_eq!(s, "GetSysStateResponse".to_string());
    }

    #[test]
    fn test_response_name_token() {
        let command_name = CommandName::from("get_sys_state");
        let response_struct_name = ResponseStructName::from(&command_name);
        let tokens: rust::Tokens = quote! {$(response_struct_name)};

        assert_eq!("GetSysStateResponse\n", tokens.to_file_string().unwrap());
    }
}
