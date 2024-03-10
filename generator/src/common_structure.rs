use serde::Deserialize;

use std::collections::HashMap;

use crate::field::Field;

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct CommonStructure {
    /// If no fields are specified then just treat the register as a set of bits
    fields: Option<HashMap<String, Field>>,
}
