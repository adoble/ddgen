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
        // #[serde(rename = "enum")]
        // enumeration: Option<String>,
        // #[serde(rename = "type", default)]
        #[serde(rename = "type")]
        #[serde(deserialize_with = "from_type_spec")]
        #[serde(default)]
        target_type: Option<TargetType>,
        description: Option<String>,
    },
}

#[derive(Deserialize, Debug, Default, Clone)]
pub enum TargetType {
    #[default]
    U8,
    U16,
    U32,
    U64,
    U128,
    I8,
    I16,
    I32,
    I64,
    I128,
    // The string is the name of the enumeration
    Enumeration(String),
}

impl From<String> for TargetType {
    fn from(value: String) -> Self {
        match value.as_str() {
            "u8" => TargetType::U8,
            "u16" => TargetType::U16,
            "u32" => TargetType::U32,
            "u64" => TargetType::U64,
            "u128" => TargetType::U128,
            "i8" => TargetType::I8,
            "i16" => TargetType::I16,
            "i32" => TargetType::I32,
            "i64" => TargetType::I64,
            "i128" => TargetType::I128,
            _ => TargetType::Enumeration(value),
        }
    }
}

impl Into<String> for TargetType {
    fn into(self) -> String {
        match self {
            TargetType::U8 => "u8".to_string(),
            TargetType::U16 => "u16".to_string(),
            TargetType::U32 => "u32".to_string(),
            TargetType::U64 => "u64".to_string(),
            TargetType::U128 => "u128".to_string(),
            TargetType::I8 => "i8".to_string(),
            TargetType::I16 => "i16".to_string(),
            TargetType::I32 => "i32".to_string(),
            TargetType::I64 => "i64".to_string(),
            TargetType::I128 => "i128".to_string(),
            // TODO this seems rather akward
            TargetType::Enumeration(name) => {
                let target_type = name.to_string().to_case(Case::UpperCamel);
                target_type
            }
        }
    }
}

fn from_bit_spec<'de, D>(deserializer: D) -> Result<BitSpec, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;

    bit_lang::parse(s.as_str()).map_err(D::Error::custom)
}

fn from_type_spec<'de, D>(deserializer: D) -> Result<Option<TargetType>, D::Error>
where
    D: Deserializer<'de>,
{
    // let s: String = Deserialize::deserialize(deserializer)?;

    let t: String = Deserialize::deserialize(deserializer)?;
    let target_type = TargetType::from(t);
    Ok(Some(target_type))
}

enum FunctionType {
    Enumeration(String, String), // paramter name, parameter type
    SingleBit,
    Value(String), // parameter_name
}

impl Field {
    pub fn generate_field(&self, tokens: &mut Tokens<Rust>, name: &str) {
        // Add field comment

        match self {
            Field::Structure {
                common_structure_name,
            } => todo!(),
            Field::BitField {
                bit_range,
                target_type,
                description,
            } => {
                // Description
                if description.is_some() {
                    let comments = DocComment::from_string(description.as_deref().unwrap());
                    quote_in!(*tokens =>
                        $(comments.as_string())
                        $['\r']
                    );
                }

                let type_string = match target_type {
                    Some(t) => t.clone().into(),
                    None => "u8".to_string(),
                };

                // Field name
                quote_in!(*tokens =>
                    $(name): $(type_string),$['\r']
                );
            }
        }
    }
}
