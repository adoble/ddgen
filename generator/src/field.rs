use std::{cmp::Ordering, collections::HashMap, fmt};

use convert_case::{Case, Casing};
use genco::prelude::*;
use serde::{de::Error, Deserialize, Deserializer};

use crate::common_structure::CommonStructure;
use crate::doc_comment::DocComment;
use crate::members::Members;
use bit_lang::{bit_spec::WordRange, BitRange, BitSpec, Repeat, Word};

#[derive(Deserialize, Debug, Eq)]
#[serde(deny_unknown_fields)]
#[serde(untagged)]
pub enum Field {
    Structure {
        #[serde(rename = "struct")]
        common_structure_name: String,
        description: Option<String>,
        #[serde(rename = "bits")]
        #[serde(deserialize_with = "from_bit_spec")]
        bit_spec: BitSpec,
    },
    BitField {
        #[serde(rename = "bits")]
        #[serde(deserialize_with = "from_bit_spec")]
        bit_spec: BitSpec,

        #[serde(rename = "type")]
        #[serde(deserialize_with = "from_type_spec")]
        #[serde(default)]
        target_type: Option<TargetType>,
        description: Option<String>,
    },
}

impl Ord for Field {
    fn cmp(&self, other: &Self) -> Ordering {
        let bit_spec = match self {
            Field::Structure { bit_spec, .. } => bit_spec,
            Field::BitField { bit_spec, .. } => bit_spec,
        };

        let bit_spec_other = match other {
            Field::Structure { bit_spec, .. } => bit_spec,
            Field::BitField { bit_spec, .. } => bit_spec,
        };

        bit_spec.cmp(bit_spec_other)
    }
}

impl PartialOrd for Field {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Field {
    fn eq(&self, other: &Self) -> bool {
        let bit_spec = match self {
            Field::Structure { bit_spec, .. } => bit_spec,
            Field::BitField { bit_spec, .. } => bit_spec,
        };

        let bit_spec_other = match other {
            Field::Structure { bit_spec, .. } => bit_spec,
            Field::BitField { bit_spec, .. } => bit_spec,
        };

        bit_spec == bit_spec_other
    }
}

// TODO merge this with BitSpecType in  BitSpec.
#[derive(Deserialize, Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord)]
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
    // Enumeration(String),
    // The string is the name of the provider
    // Provider(String),
    TypeName(String),
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
            //_ => TargetType::Enumeration(value),
            _ => TargetType::TypeName(value),
        }
    }
}

// // Need a seperate Into::into as the conversion is not symetric and cannot
// // be automatically handled by the compiler.
// #[allow(clippy::from_over_into)]
// impl Into<String> for TargetType {
//     fn into(self) -> String {
//         match self {
//             TargetType::U8 => "u8".to_string(),
//             TargetType::U16 => "u16".to_string(),
//             TargetType::U32 => "u32".to_string(),
//             TargetType::U64 => "u64".to_string(),
//             TargetType::U128 => "u128".to_string(),
//             TargetType::I8 => "i8".to_string(),
//             TargetType::I16 => "i16".to_string(),
//             TargetType::I32 => "i32".to_string(),
//             TargetType::I64 => "i64".to_string(),
//             TargetType::I128 => "i128".to_string(),
//             // TODO this seems rather akward
//             TargetType::TypeName(name) => name.to_string().to_case(Case::UpperCamel),
//             // TargetType::Enumeration(name) => name.to_string().to_case(Case::UpperCamel),
//             // TargetType::Provider(name) => name.to_string().to_case(Case::UpperCamel),
//         }
//     }
// }

impl fmt::Display for TargetType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let display_str = match self {
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
            TargetType::TypeName(name) => name.to_string().to_case(Case::UpperCamel),
            // TargetType::Enumeration(name) => name.to_string().to_case(Case::UpperCamel),
            // TargetType::Provider(name) => name.to_string().to_case(Case::UpperCamel),
        };
        write!(f, "{display_str}",)
    }
}

type CommonStructures = HashMap<String, CommonStructure>;

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

impl Field {
    pub fn generate_struct_member(&self, tokens: &mut Tokens<Rust>, name: &str) {
        match self {
            Field::Structure {
                common_structure_name,
                description,
                ..
            } => {
                if description.is_some() {
                    let comments = DocComment::from_string(description.as_deref().unwrap());
                    quote_in!(*tokens =>
                        $(comments.as_string())
                        $['\r']
                    );
                }

                let normalised_common_struct_name = common_structure_name.to_case(Case::UpperCamel);

                quote_in!(*tokens =>
                    $(name): $(normalised_common_struct_name),$['\r']
                );
            }
            Field::BitField {
                target_type,
                description,
                bit_spec,
            } => {
                if description.is_some() {
                    let comments = DocComment::from_string(description.as_deref().unwrap());
                    quote_in!(*tokens =>
                        $(comments.as_string())
                        $['\r']
                    );
                }

                let type_string = match target_type {
                    Some(t) => t.clone().to_string(),
                    None => bit_spec.suggested_word_type(),
                };

                let type_string = match bit_spec.repeat {
                    Repeat::Fixed { number } => format!("[{}; {}]", type_string, number),
                    Repeat::Dependent { limit, .. } => {
                        format!("[{}; {}]", type_string, limit)
                    }
                    Repeat::Variable { limit: _ } => match target_type {
                        Some(target_type) => format!("{}", target_type),
                        None => panic!("Variable repeat for {name} with no provider struct (in the type attribute) specified!"),
                    },
                    Repeat::None => type_string,
                };

                // Field name
                quote_in!(*tokens =>
                    $(name): $(type_string),$['\r']
                );
            }
        }
    }

    pub fn generate_field_serialization(
        &self,
        tokens: &mut Tokens<Rust>,
        name: &str,
        members: &Members,
        common_structures: &CommonStructures,
    ) {
        let field_serialize_code = match self {
            Field::BitField { bit_spec: _, .. } => {
                //self.generate_word_field_serialization(name, bit_spec, members)
                self.generate_word_field_serialization(name, members)
            }

            Field::Structure {
                common_structure_name,
                ..
            } => self.generate_header_field_serialization(
                name,
                common_structure_name,
                common_structures,
            ),
        };

        quote_in!(*tokens =>
            $(field_serialize_code);$['\r']
        );
    }
    pub fn generate_field_deserialization(
        &self,
        tokens: &mut Tokens<Rust>,
        name: &str,
        //symbol_table: &HashMap<BitSpec, String>,
        members: &Members,
    ) {
        let field_deserialize_code = match self {
            Field::BitField { bit_spec, .. } => {
                self.generate_word_field_deserialization(name, bit_spec, members)
            }

            Field::Structure {
                common_structure_name,
                ..
            } => self.generate_header_field_deserialization(common_structure_name),
        };

        quote_in!(*tokens =>
            $(field_deserialize_code);$['\r']
        );
    }

    fn generate_word_field_serialization(
        &self,
        name: &str,
        //bit_spec: &BitSpec,
        members: &Members,
    ) -> String {
        // match bit_spec {
        let bit_spec = self.bit_spec();
        match bit_spec {
            BitSpec {
                start:
                    Word {
                        index,
                        bit_range: BitRange::Single(bit_position),
                    },
                end: None,
                repeat: Repeat::None,
            } => format!("data[{index}].serialize_bit(self.{name}, {bit_position})"),
            BitSpec {
                start:
                    Word {
                        index,
                        bit_range: BitRange::Range(start_bit, end_bit),
                    },
                end: None,
                repeat: Repeat::None,
            } => {
                format!("data[{index}].serialize_field(self.{name} as u8, {start_bit}, {end_bit})")
            }
            BitSpec {
                start:
                    Word {
                        index,
                        bit_range: BitRange::WholeWord,
                    },
                end: None,
                repeat: Repeat::None,
            } => format!("data[{index}].serialize_word(self.{name})"),
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
            } => format!("data[{start_index}..={end_index}].serialize_word(self.{name})"),

            BitSpec {
                start:
                    Word {
                        index: _,
                        bit_range: BitRange::WholeWord,
                    },

                repeat: Repeat::Fixed { number },
                ..
            } => {
                let WordRange::Fixed(start_index, end_index) = bit_spec.word_range() else {
                    panic!("Repeating bit specification should have been a fixed repeat")
                };
                format!("data[{start_index}..={end_index}].serialize_repeating_words(self.{name}, {number})")
                //format!("data[{start_index}..].serialize_repeating_words(self.{name}, {limit})")
            }
            BitSpec {
                start:
                    Word {
                        index: start_index,
                        bit_range: BitRange::WholeWord,
                    },

                repeat:
                    Repeat::Dependent {
                        word: repeat_word, ..
                    },
                ..
            } => {
                // The parser assumes that the repeat word is a simple word (i.e. no range, no repeats) -
                // for example, a byte.
                // This excludes repeat words that are, for instance, a u16 that using two words.
                // What follows is a workaround, but ultimately the parser
                //  should recognise full bit specs for variable repeat words.
                let repeat_bit_spec = BitSpec {
                    start: repeat_word.clone(),
                    end: None,
                    repeat: Repeat::None,
                };
                // let count_symbol_name = symbol_table.get(&repeat_bit_spec);
                if let Some((count_symbol_name, _)) =
                    members.find_field_by_bitspec(&repeat_bit_spec)
                {
                    format!(
                        "data[{start_index}..].serialize_repeating_words(self.{}, self.{} as usize)",
                        name,
                        count_symbol_name
                    )
                } else {
                    // TODO This is a fatal error
                    format!("Cannot find bit spec {bit_spec}")
                }
            }
            BitSpec {
                repeat: Repeat::Variable { .. },
                ..
            } => {
                format!("let provider = self.{name}")
            }
            BitSpec {
                start:
                    Word {
                        index: start_index,
                        bit_range: BitRange::Literal(literal),
                    },
                ..
            } => format!("data[{start_index}] = {literal}"),
            _ => format!("todo!(\"{name}\")"),
        }
    }

    fn generate_word_field_deserialization(
        &self,
        name: &str,
        bit_spec: &BitSpec,
        //symbol_table: &HashMap<BitSpec, String>,
        members: &Members,
    ) -> String {
        match bit_spec {
            BitSpec {
                start:
                    Word {
                        index,
                        bit_range: BitRange::Single(bit_position),
                    },
                end: None,
                repeat: Repeat::None,
                //} => format!("data[{index}].serialize_bit(self.{name}, {bit_position});"),
            } => format!("self[{index}].deserialize_bit({bit_position})"),
            BitSpec {
                start:
                    Word {
                        index,
                        bit_range: BitRange::Range(start_bit, end_bit),
                    },
                end: None,
                repeat: Repeat::None,
            } => {
                format!("self[{index}].deserialize_field({start_bit}, {end_bit}).try_into()?")
            }
            BitSpec {
                start:
                    Word {
                        index,
                        bit_range: BitRange::WholeWord,
                    },
                end: None,
                repeat: Repeat::None,
            } => format!("self[{index}].deserialize_word()"),
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
            } => format!("self[{start_index}..={end_index}].deserialize_word()"),

            BitSpec {
                start:
                    Word {
                        index: start_index,
                        bit_range: BitRange::WholeWord,
                    },

                repeat: Repeat::Fixed { number },
                ..
            } => {
                format!("self[{start_index}..].deserialize_repeating_words({number})")
            }
            BitSpec {
                start:
                    Word {
                        index: start_index,
                        bit_range: BitRange::WholeWord,
                    },

                repeat:
                    Repeat::Dependent {
                        word: repeat_word, ..
                    },
                ..
            } => {
                // The parser assumes that the repeat word is a simple word (i.e. no range, no repeats) -
                // for example, a byte.
                // This excludes repeat words that are, for instance, a u16 that using two words.
                // In most cases this is enough, however the symbol table maps full bit_specs to symbols as
                // it needs to cover all symbols. What follows is a workaround, but ultimately the parser
                //  should recognise full bit specs for variable repeat words.
                // TODO do we need the symbol_table?

                if let Some((count_symbol_name, _)) =
                    members.find_field_by_bitspec(&BitSpec::from_word(repeat_word))
                {
                    format!(
                        //"self[{start_index}..].deserialize_repeating_words(self[{}].deserialize_word() as usize)",
                        "self[{start_index}..].deserialize_repeating_words({} as usize)",
                        //repeat_word.index
                        count_symbol_name
                    )
                } else {
                    println!("Cannot find bit_spec {}", bit_spec);
                    todo!("Proper error handling");
                }
            }
            BitSpec {
                start:
                    Word {
                        index: _start_index,
                        bit_range: BitRange::Literal(literal),
                    },
                ..
            } => literal.to_string(), //format!("{literal}"),
            _ => format!("todo!(\"{name}\")"),
        }
    }

    fn generate_header_field_serialization(
        &self,
        field_name: &str,
        common_structure_name: &str,
        common_structures: &CommonStructures,
    ) -> String {
        // e.g data[0..=1].serialize_struct::<2>(self.status);

        // Get the size of the common structure. Note that common structures can only have a fixed size.

        let common_structure = common_structures
            .get(common_structure_name)
            //.expect(format!("Fatal Error: Common structure {common_structure_name} is not found! Check that the names of common structures are the same.").as_str());
            .unwrap_or_else(|| panic!("Fatal Error: Common structure {common_structure_name} is not found! Check that the names of common structures are the same."));
        let size = common_structure.size();

        // Position of the common structure
        let bit_spec = self.bit_spec();
        let start = format!("{}", &bit_spec.start.index);
        let end = bit_spec
            .clone()
            .end
            .map(|w| format!("..={}", w.index))
            .unwrap_or_default();

        //         data[0..=1].serialize_struct::<2>(self.status);
        format!("data[{start}{end}].serialize_struct::<{size}>(self.{field_name})")

        //format!("todo!(\"Complete generate_header_field_serialization\")")
    }

    fn generate_header_field_deserialization(&self, common_structure_name: &str) -> String {
        // data.deserialize().unwrap()

        let bit_spec = self.bit_spec();

        let WordRange::Fixed(start, end) = bit_spec.word_range() else {
            panic!(
                "Common structure {} should have fixed size",
                common_structure_name
            );
        };

        format!("self[{start}..{end}].deserialize().unwrap()")
    }

    fn bit_spec(&self) -> &BitSpec {
        match self {
            Field::Structure { bit_spec, .. } => bit_spec,
            Field::BitField { bit_spec, .. } => bit_spec,
        }
    }

    /// If the field has a provider type than return the name of it.
    pub fn provider(&self) -> Option<&str> {
        match self {
            Field::BitField {
                bit_spec:
                    BitSpec {
                        repeat: Repeat::Variable { .. },
                        ..
                    },
                target_type: Some(TargetType::TypeName(provider_name)),
                ..
            } => Some(provider_name),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bit_lang::parse;
    #[test]
    fn test_order() {
        let lower = Field::BitField {
            bit_spec: parse("1[]").unwrap(),
            target_type: None,
            description: None,
        };

        let upper = Field::BitField {
            bit_spec: parse("2[]").unwrap(),
            target_type: None,
            description: None,
        };

        assert!(upper > lower);

        let lower = Field::BitField {
            bit_spec: parse("1[]").unwrap(),
            target_type: Some(TargetType::I16),
            description: None,
        };

        let upper = Field::BitField {
            bit_spec: parse("2[]").unwrap(),
            target_type: Some(TargetType::U16),
            description: None,
        };

        assert!(upper > lower);

        let lower = Field::BitField {
            bit_spec: parse("1[]").unwrap(),
            target_type: Some(TargetType::U16),
            description: None,
        };

        let upper = Field::BitField {
            bit_spec: parse("1[]").unwrap(),
            target_type: Some(TargetType::I16),
            description: None,
        };

        assert!(upper == lower);

        let lower = Field::BitField {
            bit_spec: parse("1[]").unwrap(),
            target_type: Some(TargetType::U16),
            description: Some("ZZZZ".to_string()),
        };

        let upper = Field::BitField {
            bit_spec: parse("1[]").unwrap(),
            target_type: Some(TargetType::I16),
            description: Some("AAAA".to_string()),
        };

        assert!(upper == lower);
    }

    #[test]
    fn test_generate_struct_member_bool() {
        let field = Field::new_bitfield("4", None).unwrap();
        let mut tokens = rust::Tokens::new();

        field.generate_struct_member(&mut tokens, "a_bool");

        assert_eq!(tokens.to_string().unwrap(), "a_bool: bool,");
    }

    #[test]
    fn test_generate_struct_member_field() {
        let field = Field::new_bitfield("5[1..4]", Some("AnEnum")).unwrap();
        let mut tokens = rust::Tokens::new();

        field.generate_struct_member(&mut tokens, "a_bool");

        assert_eq!(tokens.to_string().unwrap(), "a_bool: AnEnum,");
    }

    #[test]
    fn test_generate_struct_member_u8() {
        let field = Field::new_bitfield("3[]", None).unwrap();
        let mut tokens = rust::Tokens::new();

        field.generate_struct_member(&mut tokens, "a_u8");

        assert_eq!(tokens.to_string().unwrap(), "a_u8: u8,");
    }

    #[test]
    fn test_generate_struct_member_u16() {
        let field = Field::new_bitfield("3[]..4[]", None).unwrap();
        let mut tokens = rust::Tokens::new();

        field.generate_struct_member(&mut tokens, "a_u16");

        assert_eq!(tokens.to_string().unwrap(), "a_u16: u16,");
    }
    #[test]
    fn test_generate_struct_member_u32() {
        let field = Field::new_bitfield("3[]..6[]", None).unwrap();
        let mut tokens = rust::Tokens::new();

        field.generate_struct_member(&mut tokens, "a_u32");

        assert_eq!(tokens.to_string().unwrap(), "a_u32: u32,");
    }

    #[test]
    fn test_generate_struct_member_fixed_repeat_u8() {
        let field = Field::new_bitfield("5[];10", None).unwrap();
        let mut tokens = rust::Tokens::new();

        field.generate_struct_member(&mut tokens, "a_repeat");

        assert_eq!(tokens.to_string().unwrap(), "a_repeat: [u8; 10],");
    }

    #[test]
    fn test_generate_struct_member_fixed_repeat_u16() {
        let field = Field::new_bitfield("5[]..6[];12", None).unwrap();
        let mut tokens = rust::Tokens::new();

        field.generate_struct_member(&mut tokens, "a_repeat");

        assert_eq!(tokens.to_string().unwrap(), "a_repeat: [u16; 12],");
    }

    #[test]
    fn test_generate_struct_member_dependent_repeat_u16() {
        let field = Field::new_bitfield("5[]..6[];(1[])<12", None).unwrap();
        let mut tokens = rust::Tokens::new();

        field.generate_struct_member(&mut tokens, "a_dependent_repeat");

        assert_eq!(
            tokens.to_string().unwrap(),
            "a_dependent_repeat: [u16; 11],"
        );
    }

    #[test]
    fn test_generate_struct_member_variable_repeat() {
        let field = Field::new_bitfield("5[]..6[];<=1024", Some("ProviderStruct")).unwrap();

        let mut tokens = rust::Tokens::new();

        field.generate_struct_member(&mut tokens, "a_variable_repeat");

        assert_eq!(
            tokens.to_string().unwrap(),
            "a_variable_repeat: ProviderStruct,"
        );
    }

    #[test]
    fn test_generate_word_field_serialization_bit() {
        let field = Field::new_bitfield("5[7]", None).unwrap();

        let fields = Members::new();
        let s = field.generate_word_field_serialization("a_bit", &fields);

        assert_eq!(s, "data[5].serialize_bit(self.a_bit, 7)");
    }

    #[test]
    fn test_generate_word_field_serialization_field() {
        let field = Field::new_bitfield("5[1..4]", Some("AnEnum")).unwrap();

        let fields = Members::new();
        let s = field.generate_word_field_serialization("a_field", &fields);

        assert_eq!(s, "data[5].serialize_field(self.a_field as u8, 1, 4)");
    }

    #[test]
    fn test_generate_word_u8_serialization() {
        let field = Field::new_bitfield("5[]", None).unwrap();

        let fields = Members::new();
        let s = field.generate_word_field_serialization("a_u8", &fields);

        assert_eq!(s, "data[5].serialize_word(self.a_u8)");
    }

    #[test]
    fn test_generate_word_u16_serialization() {
        let field = Field::new_bitfield("5[]..6[]", None).unwrap();

        let fields = Members::new();
        let s = field.generate_word_field_serialization("a_u16", &fields);

        assert_eq!(s, "data[5..=6].serialize_word(self.a_u16)");
    }

    #[test]
    fn test_generate_word_fixed_repeat_u8() {
        let field = Field::new_bitfield("5[];10", None).unwrap();

        let fields = Members::new();
        let s = field.generate_word_field_serialization("a_repeat_u8", &fields);

        assert_eq!(
            s,
            "data[5..=14].serialize_repeating_words(self.a_repeat_u8, 10)"
        );
    }

    #[test]
    fn test_generate_word_fixed_repeat_u32() {
        let field = Field::new_bitfield("5[]..8[];10", None).unwrap();

        let fields = Members::new();
        let s = field.generate_word_field_serialization("a_repeat_u32", &fields);

        assert_eq!(
            s,
            "data[5..=44].serialize_repeating_words(self.a_repeat_u32, 10)"
        );
    }

    #[test]
    fn test_generate_word_dependent_repeat_u8() {
        let field = Field::new_bitfield("3[];(2[])<=10", None).unwrap();

        let mut fields = Members::new();
        let dependent_field = Field::new_bitfield("2[]", None).unwrap();
        fields.add("count", dependent_field);
        let s = field.generate_word_field_serialization("a_repeat_u32", &fields);

        assert_eq!(
            s,
            "data[3..].serialize_repeating_words(self.a_repeat_u32, self.count as usize)"
        );
    }

    #[test]
    fn test_generate_word_variable_repeat_u8() {
        let field = Field::new_bitfield("3[];<=10", None).unwrap();

        let fields = Members::new();

        let s = field.generate_word_field_serialization("a_repeat_u32", &fields);

        assert_eq!(s, "let provider = self.a_repeat_u32");
    }
    #[test]
    fn test_provider() {
        let field = Field::new_bitfield("3[];<=10", Some("a_provider")).unwrap();

        let provider = field.provider();

        assert_eq!(provider, Some("a_provider"));

        let field = Field::new_bitfield("3[3..5]", Some("an_enum")).unwrap();

        let provider = field.provider();

        assert_eq!(provider, None);

        let field = Field::new_bitfield("3];5", None).unwrap();

        let provider = field.provider();

        assert_eq!(provider, None);

        let field = Field::new_bitfield("3];<5", None).unwrap();

        let provider = field.provider();

        assert_eq!(provider, None);
    }

    // Test utilities
    impl Field {
        fn new_bitfield(bit_spec: &str, target_type: Option<&str>) -> Result<Field, String> {
            // TODO proper error handling here. This requires that the error handling in the
            // generator is completely overhauled.
            let bit_spec = parse(bit_spec).map_err(|e| e.to_string())?;

            let target_type: Option<TargetType> = match target_type {
                Some(tt) => Some(TargetType::from(tt.to_string())),
                None => None,
            };

            Ok(Field::BitField {
                bit_spec,
                target_type,
                description: None,
            })
        }
    }
}
