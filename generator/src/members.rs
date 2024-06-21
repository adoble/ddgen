use bit_lang::bit_spec::WordRange;
use bit_lang::BitSpec;
use genco::prelude::*;
use serde::Deserialize;
use std::collections::HashMap;

use crate::common_structure::CommonStructure;
use crate::field::Field;

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Members(HashMap<String, Field>);

type CommonStructures = HashMap<String, CommonStructure>;
type FieldName = String;
type MembersSize = (usize, Vec<(usize, FieldName)>);

impl Members {
    #[cfg(test)]
    pub fn new() -> Members {
        Members(HashMap::new())
    }

    #[cfg(test)]
    pub fn add(&mut self, name: &str, field: Field) {
        self.0.insert(name.to_string(), field);
    }

    pub fn generate_members(&self, tokens: &mut Tokens<Rust>) {
        let mut sorted_members = self.to_vec();

        // Sort by fields, not by the name
        sorted_members.sort_by(|(_, field_a), (_, field_b)| field_a.cmp(field_b));

        for (name, field) in sorted_members {
            field.generate_struct_member(tokens, name);
        }
    }

    pub fn generate_serializations(
        &self,
        tokens: &mut Tokens<Rust>,
        struct_name: &str,
        common_structures: &CommonStructures,
    ) {
        let mut sorted_members: Vec<_> = self.to_vec();

        // Sort by fields, not by the name. This should give an order that is closer to what woudl be in a
        // device data sheet. It is also essential for an integration testing as code generation should
        // always give the same results.
        sorted_members.sort_by(|(_, field_a), (_, field_b)| field_a.cmp(field_b));

        // Generate the expresssion required to calculate the actual nmber of bytes serialized
        let serialization_size_expression = self.generate_serialization_size_expression();

        quote_in!(*tokens =>
            impl Serialize for $(struct_name) {
                fn serialize<const N: usize>(&self) -> (usize, [u8; N], impl Iterator<Item=u8>) {
                  let mut data = [0u8; N];
                  #[allow(unused_variables)]
                  let provider = core::iter::empty::<u8>();

                  $(for (name, field) in sorted_members => $(ref toks {field.generate_field_serialization(toks,  name,  self, common_structures)}) )

                  ($(serialization_size_expression), data, provider)
                }


            }
        );
    }

    /// Generate the expresssion required to calculate the actual number of bytes serialized
    fn generate_serialization_size_expression(&self) -> String {
        let (fixed_size, variable_sizes) = self.size();

        let variable_size_expression: String = variable_sizes
            .iter()
            .map(|s| format!(" + ({} * self.{} as usize)", s.0, s.1))
            .fold(String::new(), |acc, s| acc + s.as_str());

        format!("{fixed_size}{variable_size_expression}")
    }

    pub fn generate_deserializations(&self, tokens: &mut Tokens<Rust>, struct_name: &str) {
        let mut sorted_members: Vec<_> = self.to_vec();

        // Sort by fields, not by the name
        sorted_members.sort_by(|(_, field_a), (_, field_b)| field_a.cmp(field_b));

        quote_in!(*tokens=>
           impl Deserialize<Self> for $(struct_name) {

               fn deserialize(buf: &[u8]) -> Result<$(struct_name), DeviceError> { $['\r']

                    $(for (name, field) in &sorted_members => let $(*name) = $(ref toks {field.generate_field_deserialization(toks,  name, self)}) ) $['\r']

                    Ok(Self {$['\r']
                        $(for (name, _) in &sorted_members => $(*name),$['\r'])
                    })$['\r']


               }$['\r']

           }$['\r']
        );
    }

    /// Calculates the max size in bytes of a set of members. This is required
    /// so that the buffers for the structures can be sized to cater for
    /// largest size.
    #[allow(dead_code)]
    pub fn buffer_size(&self, common_structures: &HashMap<String, CommonStructure>) -> usize {
        let mut positions: HashMap<usize, usize> = HashMap::new();
        for f in self.fields() {
            // Implementing this with a hashmap as more than one bit spec can reference the
            // same position in the buffer. The key is the position and the value is the size.
            match f {
                Field::BitField {
                    bit_spec: bit_range,
                    ..
                } => {
                    positions
                        .entry(bit_range.start.index)
                        .or_insert_with(|| bit_range.max_size());
                }
                Field::Structure {
                    common_structure_name,
                    bit_spec,
                    ..
                } => {
                    let common_structure = common_structures.get(common_structure_name).unwrap();
                    positions
                        .entry(bit_spec.start.index)
                        .or_insert_with(|| common_structure.buffer_size());
                }
            }
        }

        positions.values().sum::<usize>()
    }

    pub fn to_vec(&self) -> Vec<(&String, &Field)> {
        let v = self.0.iter().collect();
        v
    }

    pub fn fields(&self) -> impl Iterator<Item = &Field> {
        self.0.values()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&String, &Field)> {
        self.0.iter()
    }

    pub fn size(&self) -> MembersSize {
        let fixed_size = self
            .iter()
            .filter_map(|f| match f.1 {
                Field::Structure { .. } => None,
                Field::BitField { bit_spec, .. } => Some((f.0, bit_spec)),
            })
            .filter_map(|f| {
                if let WordRange::Fixed(_start, end) = f.1.word_range() {
                    Some(end + 1)
                } else {
                    None
                }
            })
            .max()
            .unwrap_or(0);

        let variable_sizes: Vec<(usize, String)> = self
            .iter()
            .filter_map(|f| match f.1 {
                Field::Structure { .. } => None,
                Field::BitField { bit_spec, .. } => bit_spec.size(),
            })
            .filter_map(|s| match s.1 {
                Some(word) => Some((s.0, word)),
                None => None,
            })
            .map(|s| (s.0, BitSpec::from_word(&s.1)))
            .map(|s| (s.0, self.find_field_by_bitspec(&s.1).unwrap()))
            .map(|s| (s.0, s.1 .0.to_string()))
            //.map(|s| format!(" + ({} * self.{} as usize)", s.0, s.1))
            .collect();

        (fixed_size, variable_sizes)
    }

    pub fn find_field_by_bitspec(&self, bit_spec: &BitSpec) -> Option<(&str, &Field)> {
        // Using a simple linear search as there
        let mut found_field = Option::None;
        for (name, field) in self.iter() {
            let field_bit_spec = match field {
                Field::BitField { bit_spec, .. } => bit_spec,
                Field::Structure { bit_spec, .. } => bit_spec,
            };
            if field_bit_spec == bit_spec {
                found_field = Some((name.as_str(), field));
                break;
            }
        }
        found_field
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    use bit_lang::parse;
    #[test]
    fn test_find_field_by_bitspec() {
        let bit_spec0 = parse("1[]").unwrap();
        let bit_spec1 = parse("2[]..3[]").unwrap();
        let bit_spec2 = parse("4[]..5[];6").unwrap();

        let field0 = Field::BitField {
            bit_spec: bit_spec0,
            target_type: None,
            description: None,
        };
        let field1 = Field::BitField {
            bit_spec: bit_spec1.clone(),
            target_type: None,
            description: None,
        };
        let field2 = Field::BitField {
            bit_spec: bit_spec2,
            target_type: None,
            description: None,
        };

        let map = HashMap::from([
            ("a_u8".to_string(), field0),
            ("a_u16".to_string(), field1),
            ("a_repeating_u16".to_string(), field2),
        ]);

        let members = Members(map);

        let expected_bit_spec = parse("2[]..3[]").unwrap();
        let expected_field = Field::BitField {
            bit_spec: expected_bit_spec,
            target_type: None,
            description: None,
        };
        assert_eq!(
            members.find_field_by_bitspec(&bit_spec1),
            Some(("a_u16", &expected_field))
        );
    }
}
