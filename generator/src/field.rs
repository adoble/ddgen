use convert_case::{Case, Casing};
use genco::prelude::*;
use serde::{de::Error, Deserialize, Deserializer};

use crate::doc_comment::DocComment;
use bit_lang::{BitRange, BitSpec, Repeat, Word};

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

#[derive(Deserialize, Debug, PartialEq)]
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
        // // The symbolic name of the field. This is the field name assigned to in the toml
        // #[serde(skip_deserializing)]
        // symbolic_name: Option<String>,

        // // The field used to specify the  the number of repeats.
        // #[serde(skip_deserializing)]
        // repeating_number_field: Option<Box<Field>>,
    },
}

impl Field {
    pub fn is_bitfield(&self) -> bool {
        matches!(self, Field::BitField { .. })
    }

    pub fn is_structure(&self) -> bool {
        matches!(self, Field::Structure { .. })
    }
}

#[derive(Deserialize, Debug, Default, Clone, PartialEq)]
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

// Need a seperate Into::into as the conversion is not symetric and cannot
// be automatically handled by the compiler.
#[allow(clippy::from_over_into)]
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
            TargetType::Enumeration(name) => name.to_string().to_case(Case::UpperCamel),
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
    pub fn generate_struct_member(&self, tokens: &mut Tokens<Rust>, name: &str) {
        // Add field comment

        match self {
            Field::Structure {
                common_structure_name: _,
            } => todo!(),
            Field::BitField {
                target_type,
                description,
                bit_range,
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

                let type_string = match bit_range.repeat {
                    Repeat::Fixed(limit) => format!("[{}; {}]", type_string, limit),
                    Repeat::Variable { limit, .. } => format!("[{}; {}]", type_string, limit),
                    Repeat::None => type_string,
                };

                // Field name
                quote_in!(*tokens =>
                    $(name): $(type_string),$['\r']
                );
            }
        }
    }

    pub fn generate_field_serialization(&self, tokens: &mut Tokens<Rust>, name: &str) {
        let field_serialize_code = match self {
            Field::BitField { bit_range, .. } => {
                self.generate_word_field_serialization(name, bit_range)
            }
            Field::Structure {
                common_structure_name,
            } => self.generate_header_field_serialization(common_structure_name),
        };

        quote_in!(*tokens =>
            $(field_serialize_code);$['\r']
        );
    }
    pub fn generate_field_deserialization(&self, _tokens: &mut Tokens<Rust>, _name: &str) {
        //TODO
    }

    fn generate_word_field_serialization(&self, name: &str, bit_range: &BitSpec) -> String {
        match bit_range {
            BitSpec {
                start:
                    Word {
                        index,
                        bit_range: BitRange::Single(bit_position),
                    },
                end: None,
                repeat: Repeat::None,
            } => format!("data[{index}].serialize_bit(self.{name}, {bit_position});"),
            BitSpec {
                start:
                    Word {
                        index,
                        bit_range: BitRange::Range(start_bit, end_bit),
                    },
                end: None,
                repeat: Repeat::None,
            } => format!("data[{index}].serialize_field(self.{name}, {start_bit}, {end_bit});"),
            BitSpec {
                start:
                    Word {
                        index,
                        bit_range: BitRange::WholeWord,
                    },
                end: None,
                repeat: Repeat::None,
            } => format!("data[{index}].serialize_word(self.{name});"),
            BitSpec {
                start:
                    Word {
                        index: start_index,
                        bit_range: BitRange::WholeWord,
                    },
                end:
                    Some(Word {
                        index: end_index,
                        bit_range: BitRange::WholeWord,
                    }),
                repeat: Repeat::None,
            } => format!("data[{start_index}..={end_index}].serialize_word(self.{name});"),

            _ => format!("todo!(\"{name}\")"),
        }
    }

    fn generate_header_field_serialization(&self, _common_structure_name: &str) -> String {
        todo!()
    }
}
