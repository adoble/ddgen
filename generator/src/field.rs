use convert_case::{Case, Casing};
use genco::prelude::*;
use serde::{de::Error, Deserialize, Deserializer};

use crate::access::Access;
use crate::bit_range::BitRange;
use crate::doc_comment::DocComment;
use bit_lang::BitSpec;

// #[derive(Deserialize, Debug)]
// #[serde(deny_unknown_fields)]
// pub struct Field {
//     #[serde(rename = "bits")]
//     #[serde(deserialize_with = "from_bit_spec")]
//     pub(crate) bit_range: BitSpec,
//     #[serde(default)]
//     pub(crate) access: Access,
//     #[serde(rename = "enum")]
//     pub(crate) enumeration: Option<String>,
//     pub(crate) description: Option<String>,
// }

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
#[serde(untagged)]
pub enum Field {
    Structure {
        #[serde(rename = "struct")]
        common_structure_name: String,
    },
    BitField {
        #[serde(rename = "bits")]
        #[serde(deserialize_with = "from_bit_spec")]
        bit_range: BitSpec,
        #[serde(rename = "enum")]
        enumeration: Option<String>,
        description: Option<String>,
    },
}

fn from_bit_spec<'de, D>(deserializer: D) -> Result<BitSpec, D::Error>
where
    D: Deserializer<'de>,
{
    //let s: &str = Deserialize::deserialize(deserializer)?;

    //bit_lang::parse(s).map_err(D::Error::custom)
    let s: String = Deserialize::deserialize(deserializer)?;

    bit_lang::parse(s.as_str()).map_err(D::Error::custom)
}

enum FunctionType {
    Enumeration(String, String), // paramter name, parameter type
    SingleBit,
    Value(String), // parameter_name
}

impl Field {
    pub fn generate_read_field(&self, tokens: &mut Tokens<Rust>, name: &str) {
        // let is_read_field = matches!(self.access, Access::Read | Access::ReadWrite);

        // if !is_read_field {
        //     return;
        // };

        // // let bit_range = BitRange::parse(&self.bit_range).unwrap(); // TODO error handling

        // enum FunctionType {
        //     Enumeration(String),
        //     SingleBit(String),
        //     Value(String),
        // }

        // let function_type = if let Some(enum_name) = &self.enumeration {
        //     let enum_name_ucc = enum_name.to_case(Case::UpperCamel); //format!("{}", AsUpperCamelCase(enum_name));
        //     FunctionType::Enumeration(enum_name_ucc)
        // } else if self.bit_range.is_single_bit() {
        //     FunctionType::SingleBit("bool".to_string())
        // } else {
        //     FunctionType::Value("u8".to_string())
        // };

        // // Add field comment
        // if self.description.is_some() {
        //     let comments = DocComment::from_string(self.description.as_deref().unwrap());
        //     quote_in!(*tokens =>
        //         $(comments.as_string())
        //         $['\r']
        //     );
        // }

        // match function_type {
        //     FunctionType::Enumeration(t) => {
        //         quote_in!(*tokens =>
        //             pub fn $(name.to_lowercase())(&self) -> $t {
        //                 self.field($(self.bit_range.start()), $(self.bit_range.end())).try_into().unwrap()
        //             }
        //             $['\r']
        //         );
        //     }
        //     FunctionType::SingleBit(t) => {
        //         quote_in!(*tokens =>
        //             pub fn $(name.to_lowercase())(&self) -> $t {
        //                 self.bit($(self.bit_range.start()))
        //             }
        //             $['\r']
        //         );
        //     }
        //     FunctionType::Value(t) => {
        //         quote_in!(*tokens =>
        //             pub fn $(name.to_lowercase())(&self) -> $t {
        //                 self.field($(self.bit_range.start()), $(self.bit_range.end()))
        //             }
        //             $['\r']
        //         );
        //     }
        // }
        todo!();
    }

    pub fn generate_write_field(&self, tokens: &mut Tokens<Rust>, name: &str) {
        // let is_write_field = matches!(self.access, Access::Write | Access::ReadWrite);

        // if !is_write_field {
        //     return;
        // };

        // // let bit_range = BitRange::parse(&self.bit_range).unwrap(); // TODO error handling

        // let function_type = if let Some(enum_name) = &self.enumeration {
        //     let p_name = enum_name.to_lowercase();
        //     let p_type = enum_name.to_case(Case::UpperCamel);
        //     FunctionType::Enumeration(p_name.clone(), p_type)
        // } else if self.bit_range.is_single_bit() {
        //     FunctionType::SingleBit
        // } else {
        //     FunctionType::Value(name.to_lowercase().to_string())
        // };

        // let function_name = name.to_lowercase();

        // // // Add field comment
        // // if field.description.is_some() {
        // //     let comments = format!("/// {}", field.description.as_deref().unwrap());
        // //     quote_in!(*tokens =>
        // //         $comments
        // //         $['\r']
        // //     );
        // // }

        // // Add field comment
        // if self.description.is_some() {
        //     let comments = DocComment::from_string(self.description.as_deref().unwrap());
        //     quote_in!(*tokens =>
        //         $(comments.as_string())
        //         $['\r']
        //     );
        // }

        // match function_type {
        //     FunctionType::Enumeration(p_name, p_type) => {
        //         quote_in!(*tokens =>
        //             pub fn $function_name(&mut self, $(p_name.clone()): $p_type) -> &mut W {
        //                 self.modify_field($p_name as u8, $(self.bit_range.start()), $(self.bit_range.end()));
        //                 $['\r']
        //                 self
        //             }
        //             $['\n']
        //         );
        //     }
        //     FunctionType::SingleBit => {
        //         quote_in!(*tokens =>
        //             pub fn $(name.to_lowercase())(&mut self, state: bool) -> &mut W {
        //                 self.modify_bit($(self.bit_range.start()), state);
        //                 $['\r']
        //                 self
        //             }
        //             $['\n']
        //         );
        //     }
        //     FunctionType::Value(name) => {
        //         quote_in!(*tokens =>
        //             pub fn $(name.clone())(&mut self, $(name.clone()): u8) -> &mut W {
        //                 self.modify_field($name, $(self.bit_range.start()), $(self.bit_range.end()));
        //                 $['\r']
        //                 self
        //             }
        //             $['\n']
        //         );
        //     }
        // }
        todo!();
    }
}
