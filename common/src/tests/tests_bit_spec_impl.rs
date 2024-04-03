#![cfg(test)]
use crate::bits::Bits;
//use super::*;
use crate::response::{ResponseBit, ResponseField};
use crate::{error::DeviceError, response::ResponseArray};
use bit_lang::{BitRange, BitSpec, Repeat as WordRepeat, Word};

// An enum for testing
#[derive(PartialEq, PartialOrd, Debug, Clone, Copy, Default)]
pub enum TestField {
    #[default]
    Disabled = 0,
    Enabled = 1,
    Tristate = 2,
}

impl TryFrom<u8> for TestField {
    type Error = DeviceError;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Disabled),
            1 => Ok(Self::Enabled),
            2 => Ok(Self::Tristate),
            _ => Err(DeviceError::EnumConversion),
        }
    }
}

#[test]
fn test_enum_deserialize_field() {
    let source = [0, 0b0111_0000]; // Tristate

    let r: TestField = source[1].deserialize_field(3, 4).try_into().unwrap();

    assert_eq!(r, TestField::Tristate);
}

// 3[4]
#[test]
fn deserialize_bit() {
    let spec = bit_lang::parse("3[4]").unwrap();

    let data = [
        0b0000_0000,
        0b0000_0000,
        0b0000_0000,
        0b0001_0000,
        0b0000_0000,
    ];

    let (w, n) = match spec {
        BitSpec {
            start:
                Word {
                    index,
                    bit_range: BitRange::Single(n),
                },
            end: None,
            repeat: WordRepeat::None,
        } => (index, n),
        _ => {
            assert!(false, "Unexpected bit spec found");
            return;
        }
    };

    assert_eq!(w, 3);
    assert_eq!(n, 4);

    let r = data[w].deserialize_bit(n as usize);

    assert_eq!(r, true);
}

#[test]
fn test_deserialize_field() {
    let spec = bit_lang::parse("3[4..5]").unwrap();

    let data: [u8; 5] = [
        0b0000_0000, // 0
        0b0000_0000, // 1
        0b0000_0000, // 2
        0b0101_0000, // 3   3[4..5] = 0d5
        0b0000_0000, // 4
    ];

    let (w, n, m) = match spec {
        BitSpec {
            start:
                Word {
                    index,
                    bit_range: BitRange::Range(n, m),
                },
            end: None,
            repeat: WordRepeat::None,
        } => (index, n, m),
        _ => {
            assert!(false, "Unexpected bit spec found");
            return;
        }
    };

    let test_enum: TestField = data[w]
        .deserialize_field(n as usize, m as usize)
        .try_into()
        .unwrap();

    assert_eq!(test_enum, TestField::Enabled);
}

#[test]
fn deserialize_whole_word() {
    let spec = bit_lang::parse("3[]").unwrap();

    let data = [
        0b0000_0000, // 0
        0b0000_0000, // 1
        0b0000_0000, // 2
        0b0101_0000, // 3
        0b0000_0000, // 4
    ];

    let w: usize = spec.start.index.into();

    assert_eq!(spec.start.bit_range, BitRange::WholeWord);

    let r = data[w].field(0, 7);

    assert_eq!(r, 0b0101_0000);
}

#[test]
fn deserialize_word_range_le_to_u16() {
    // Little endian
    let spec = bit_lang::parse("3[]..4[]").unwrap();

    // Number of 0d1400 in 3 and 4
    let data: [u8; 5] = [
        0b0000_0000, // 0
        0b0000_0000, // 1
        0b0000_0000, // 2
        0b0111_1000, // 3  little end
        0b0000_0101, // 4  big end
    ];
    let expected = 1400;

    let (v, w) = match spec {
        BitSpec {
            start:
                Word {
                    index: v,
                    bit_range: BitRange::WholeWord,
                },
            end:
                Some(Word {
                    index: w,
                    bit_range: BitRange::WholeWord,
                }),
            repeat: WordRepeat::None,
        } => (v, w),
        _ => {
            assert!(false, "Unexpected bit spec found");
            return;
        }
    };

    assert_eq!(v, 3);
    assert_eq!(w, 4);

    let r = u16::from_le_bytes(data[v..=w].try_into().unwrap());

    assert_eq!(r, expected);
}

#[test]
fn deserialize_word_range_fields_to_u16() {
    // Litte endian
    let spec = bit_lang::parse("3[]..4[0..3]").unwrap();

    // Number of 0d1400 in 3 and 4[0..3]
    let data: [u8; 5] = [
        0b0000_0000, // 0
        0b0000_0000, // 1
        0b0000_0000, // 2
        0b0111_1000, // 3  little end
        0b1111_0101, // 4  big end. Top four bits should be ignored
    ];
    let expected = 1400;

    let v = spec.start.index;

    let (w, m, n) = if let Some(word) = spec.end {
        match word.bit_range {
            BitRange::Range(m, n) => (word.index, m, n),
            _ => {
                assert!(false, "Unexpected variant");
                (0, 0, 0)
            }
        }
    } else {
        assert!(false, "End word not specified");
        (0, 0, 0)
    };

    let r = u16::from_le_bytes([data[v], data[w].field(m as usize, n as usize)]);

    assert_eq!(r, expected);
}

#[test]
fn deserialize_word_range_with_fields_le_to_u32() {
    // Litte endian
    let spec = bit_lang::parse("3[]..6[0..3]").unwrap();

    // Number of 0d222222222 in 3 , 4, 5, 6[0..3]
    let data = [
        0x00, // 0
        0x00, // 1
        0x00, // 2
        0x8E, // 3
        0xD7, // 4
        0x3E, // 5
        0x0D, // 6
    ];

    let expected: u32 = 222_222_222;

    let v = spec.start.index;

    let (w, m, n) = if let Some(word) = spec.end {
        match word.bit_range {
            BitRange::Range(m, n) => (word.index, m, n),
            _ => {
                assert!(false, "Unexpected variant");
                (0, 0, 0)
            }
        }
    } else {
        assert!(false, "End word not specified");
        (0, 0, 0)
    };

    assert_eq!(v, 3);
    assert_eq!(w, 6);

    let r = u32::from_le_bytes([
        data[v],
        data[v + 1],
        data[v + 2],
        data[w].field(m as usize, n as usize),
    ]);

    assert_eq!(r, expected);
}

#[test]
fn deserialize_word_repeat() {
    let spec = bit_lang::parse("3[];5").unwrap();

    // Number of 0d1400 in 3 and 4
    let data: [u8; 8] = [
        0b0000_0000, // 0
        0b0000_0000, // 1
        0b0000_0000, // 2
        1,           // 3
        2,           // 4
        3,           // 5
        4,           // 6
        5,           // 7
    ];
    let expected: [u8; 5] = [1, 2, 3, 4, 5];

    let (w, r) = match spec {
        BitSpec {
            start:
                Word {
                    index: w,
                    bit_range: BitRange::WholeWord,
                },
            end: None,
            repeat: WordRepeat::Fixed(r),
        } => (w, r),
        _ => {
            assert!(false, "Unexpected bit spec found");
            return;
        }
    };

    assert_eq!(w, 3);
    assert_eq!(r, 5);

    let d: [u8; 5] = data[w..(w + r)].deserialize_repeating_words(5);

    assert_eq!(d, expected);
}

#[test]
fn deserialize_word_variable_repeat() {
    let spec = bit_lang::parse("3[];(2[])<=5").unwrap();

    // Number of 0d1400 in 3 and 4
    let data: [u8; 8] = [
        0b0000_0000, // 0
        0b0000_0000, // 1
        4,           // 2 - COntains the count of four items
        1,           // 3
        2,           // 4
        3,           // 5
        4,           // 6
        5,           // 7
    ];
    let expected: [u8; 5] = [1, 2, 3, 4, 0];

    let (w, count_word, limit) = match spec {
        BitSpec {
            start:
                Word {
                    index: w,
                    bit_range: BitRange::WholeWord,
                },
            end: None,
            repeat:
                WordRepeat::Variable {
                    word:
                        Word {
                            index: count_index,
                            bit_range: BitRange::WholeWord,
                        },
                    limit,
                },
        } => (w, count_index, limit),
        _ => {
            assert!(false, "Unexpected bit spec found");
            return;
        }
    };

    assert_eq!(w, 3);
    assert_eq!(count_word, 2);
    assert_eq!(limit, 5);

    // bits = "3[];(2[])<=5"
    let repeats: usize = data[count_word] as usize;
    let d: [u8; 5] = data[w..(w + repeats)].deserialize_repeating_words(repeats);

    assert_eq!(d, expected);
}

#[test]
fn deserialize_word_range_u16_repeat() {
    let spec = bit_lang::parse("3[]..4[];3").unwrap();

    // Number of 0d1400 in 3 and 4
    let mut data: [u8; 10] = [
        0b0000_0000, // 0
        0b0000_0000, // 1
        3,           // 2 - Contains the count of three items double items. Not used in this test
        0,           // 3
        0,           // 4
        0,           // 5
        0,           // 6
        0,           // 7
        0,           // 8
        0xFF,        // 9 extra data
    ];
    let expected: [u16; 3] = [22_222, 55_555, 65_535];

    data[3] = expected[0].to_le_bytes()[0];
    data[4] = expected[0].to_le_bytes()[1];
    data[5] = expected[1].to_le_bytes()[0];
    data[6] = expected[1].to_le_bytes()[1];
    data[7] = expected[2].to_le_bytes()[0];
    data[8] = expected[2].to_le_bytes()[1];

    let (w, v, r) = match spec {
        BitSpec {
            start:
                Word {
                    index: w,
                    bit_range: BitRange::WholeWord,
                },
            end:
                Some(Word {
                    index: v,
                    bit_range: BitRange::WholeWord,
                }),
            repeat: WordRepeat::Fixed(r),
        } => (w, v, r),
        _ => {
            assert!(false, "Unexpected bit spec found");
            return;
        }
    };

    assert_eq!(w, 3);
    assert_eq!(v, 4);
    assert_eq!(r, 3);

    let d: [u16; 3] = data[w..(w + 2 * r)].deserialize_repeating_words(3);

    assert_eq!(d, expected);
}

#[test]
fn deserialize_word_range_u16_variable_repeat() {
    let spec = bit_lang::parse("3[]..4[];(2[])<=5").unwrap();

    // Number of 0d1400 in 3 and 4
    let mut data: [u8; 10] = [
        0b0000_0000, // 0
        0b0000_0000, // 1
        3,           // 2 - Contains the count of three items double items. Not used in this testv
        0,           // 3
        0,           // 4
        0,           // 5
        0,           // 6
        0,           // 7
        0,           // 8
        0xFF,        // 9 extra data
    ];
    let expected: [u16; 5] = [22_222, 55_555, 65_535, 0, 0];

    data[3] = expected[0].to_le_bytes()[0];
    data[4] = expected[0].to_le_bytes()[1];
    data[5] = expected[1].to_le_bytes()[0];
    data[6] = expected[1].to_le_bytes()[1];
    data[7] = expected[2].to_le_bytes()[0];
    data[8] = expected[2].to_le_bytes()[1];

    let (w, v, counter_word, limit) = match spec {
        BitSpec {
            start:
                Word {
                    index: w,
                    bit_range: BitRange::WholeWord,
                },
            end:
                Some(Word {
                    index: v,
                    bit_range: BitRange::WholeWord,
                }),
            repeat:
                WordRepeat::Variable {
                    word:
                        Word {
                            index: counter_word,
                            bit_range: BitRange::WholeWord,
                        },
                    limit,
                },
        } => (w, v, counter_word, limit),
        _ => {
            assert!(false, "Unexpected bit spec found");
            return;
        }
    };

    assert_eq!(w, 3);
    assert_eq!(v, 4);
    assert_eq!(counter_word, 2);
    assert_eq!(limit, 5);

    //"3[]..4[];(2[])<=5"
    let repeats = data[counter_word] as usize;
    let d: [u16; 5] = data[w..(w + (repeats * 2))].deserialize_repeating_words(repeats);

    assert_eq!(d, expected);
}

#[test]
fn serialise_bit() {
    let b = true;

    let spec = bit_lang::parse("3[4]").unwrap();

    let expected_data = [
        0b0000_0000,
        0b0000_0000,
        0b0000_0000,
        0b0001_0000,
        0b0000_0000,
    ];

    let mut data: [u8; 5] = [0; 5];

    let (w, n) = match spec {
        BitSpec {
            start:
                Word {
                    index,
                    bit_range: BitRange::Single(n),
                },
            end: None,
            repeat: WordRepeat::None,
        } => (index, n),
        _ => {
            assert!(false, "Unexpected bit spec found");
            return;
        }
    };

    assert_eq!(w, 3);
    assert_eq!(n, 4);

    data[w].modify_bit(n as usize, b);

    assert_eq!(data, expected_data);
}

#[test]
fn serialize_field() {
    let spec = bit_lang::parse("2[2..3]").unwrap();

    let mut data = [0u8; 5];
    data[2] = 0xF1;

    let (w, n, m) = match spec {
        BitSpec {
            start:
                Word {
                    index: w,
                    bit_range: BitRange::Range(n, m),
                },
            end: None,
            repeat: WordRepeat::None,
        } => (w, n, m),
        _ => {
            assert!(false, "Unexpected bit spec found");
            return;
        }
    };

    assert_eq!(w, 2);
    assert_eq!(n, 2);
    assert_eq!(m, 3);

    let field = TestField::Tristate;

    // data[w] = modify_field(data[w], field as u8, n, m);
    data[w].modify_field(field as u8, n as usize, m as usize);

    assert_eq!(data[2], 0b1111_1001);

    let field = TestField::Enabled;

    data[w].modify_field(field as u8, n as usize, m as usize);
    assert_eq!(data[2], 0b1111_0101);
}

#[test]
fn serialize_u16() {
    let spec = bit_lang::parse("2[]..3[]").unwrap();

    // Avoiding the BitSpec destructuring as this has already been demonstrated.

    let value: u16 = 22222; // =0x56CE

    let mut serialized_data = [0u8; 10];

    let v = spec.start.index;
    let w = spec.end.unwrap().index;

    serialized_data[v..=w].copy_from_slice(&value.to_le_bytes());

    assert_eq!(serialized_data[v], 0xCE);
    assert_eq!(serialized_data[w], 0x56);
}
