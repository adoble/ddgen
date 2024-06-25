use serde::Deserialize;
use std::collections::HashMap;

use genco::prelude::*;

use crate::members::Members;
use crate::naming::CommonStructureName;

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct CommonStructure(Members);

impl CommonStructure {
    pub fn generate(
        &self,
        tokens: &mut Tokens<Rust>,
        struct_name: &CommonStructureName,
        common_structures: &HashMap<String, CommonStructure>,
    ) {
        //let struct_name = name.to_case(Case::UpperCamel);
        quote_in!(*tokens =>
            #[derive(Debug, PartialEq, Copy, Clone, Default)]
            pub struct $(struct_name.clone()) {
                $(for (name, field) in self.0.iter() => $(ref toks {field.generate_struct_member(toks, name)}) )
            }

            $(ref toks => self.0.generate_serializations(toks, struct_name.clone(), common_structures))$['\r']

            $(ref toks => self.0.generate_deserializations(toks, struct_name))$['\r']
        );
    }

    /// Determine how many bytes this structure would need.
    /// Note: Common Structures cannot contain variable fields.
    // In the future this should
    // return a Vec of tuples - `Vec<(usize, Option<String>)>` - each containing the
    // fixed size of the repeating elements and an optional String with the symbolic name
    // of the mutiplieing field
    pub fn size(&self) -> usize {
        let size = self.0.size();
        assert!(
            size.1.is_empty(),
            "Fatal Error: Common structures should not contain varaible fields."
        );
        size.0
    }

    /// Calculates the size in bytes required to hold a common structure.
    /// Common structures cannot contain other common structures
    pub fn buffer_size(&self) -> usize {
        let empty_common_structures = HashMap::<String, CommonStructure>::new();
        self.0.buffer_size(&empty_common_structures)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::field::Field;
    use bit_lang::parse;

    #[test]
    fn test_buffer_size_simple() {
        let field_a = Field::BitField {
            bit_spec: parse("0[]").unwrap(),
            target_type: None,
            description: None,
        };
        let field_b = Field::BitField {
            bit_spec: parse("1[]").unwrap(),
            target_type: None,
            description: None,
        };
        let field_c = Field::BitField {
            bit_spec: parse("2[]").unwrap(),
            target_type: None,
            description: None,
        };

        let mut members = Members::new();

        members.add("a", field_a);
        members.add("b", field_b);
        members.add("c", field_c);

        let common_structure = CommonStructure(members);

        let buf_size = common_structure.buffer_size();

        assert_eq!(3, buf_size);
    }

    #[test]
    fn test_buffer_size_discontinuous() {
        let field_a = Field::BitField {
            bit_spec: parse("1[]").unwrap(),
            target_type: None,
            description: None,
        };

        let field_c = Field::BitField {
            bit_spec: parse("3[]").unwrap(),
            target_type: None,
            description: None,
        };

        let mut members = Members::new();

        members.add("a", field_a);
        members.add("c", field_c);

        let common_structure = CommonStructure(members);

        let buf_size = common_structure.buffer_size();

        assert_eq!(4, buf_size);
    }
}
