//! A language parser for specifying consecutive bits in a set of words.
//!
//! # Language specification
//!
//! ## Single bits
//! To refer to a single bit in a word use:
//! ```text
//! w[b]
//! ```
//! where `w` is the word index and `b` is the bit index. Both `w` and `b` are 0 based.
//!
//! For instance, to represent the bit  4in  word 3 use:
//!```text
//! 3[4]
//!```
//! Word indexes default to 0 if not specified so:
//!```text
//! [5] == 0[5]
//!```
//! ( `==` means equivant to).
//!
//! If the word is not specified then the square bracket can be omitted:
//!
//! ```text
//! 5 == [5] == 0[5]
//!```
//!
//! Note that all the above forms can be used.
//!
//! ## Bit Ranges
//!
//! To refer to a range of bits use:
//!```text
//! w[a..b]
//!```
//! where `a` is the first bit and `b` is the last bit **inclusive**.
//!
//! For instance, to refer to bit 3 to 6 inclusive in the 2 word use:
//!```text
//! 2[3..6]
//!```
//! As for single bits, if word indexes are 0 then they do not need to be specified and neither do the square brackets:
//!```text
//! 3.6 == [3..6] == 0[3..6]
//!```
//! Note that all the above forms can be used.
//!
//! ## Whole Words
//!
//! A whole word can be specifed an emtpy range:
//!```text
//! w[]
//!```
//! Refers to all thre bits in word `w`.
//!
//! For instance to refers all bits in word 5 use:
//!```text
//! 5[]
//!```
//! To refer to the whole of word `0`  use one of the following:
//!```text
//! [] == 0[]
//!```
//! ## Word Ranges
//!
//! To refer to a range of bits over more then one consecutive word use:
//!```text
//! w[a]..v[b]
//!```
//! This refers to a set of bits from bit `a` in word `w` to bit `b`in word `v`.
//!
//! Examples:
//!```text
//! 3[4]..6[2]
//!```
//! Refers to all the bits from bit 4 in word 3 to bit 2 in word 6.
//!
//!
//! As before an empty bit range refers to the whole word:
//!```text
//! 3[]..4[]
//!```
//! Refers to all the bits in word 3 and 4 (e.g a value over two words).
//!
//! As the bits specified need to be consecutive. specifiying ranges for each word in
//! a word range is not allowed. However, the following is possible:
//! ```text
//! 3[]..6[0.3]
//! ```
//! Here the consective bits in words 3 to 5 as well as the first 4 bits in word 6. This covers
//! cases where, for instance, a number is specified is less than a multiple of the word size.
//!
//! ## Repeating Words
//!
//! To specify that as word repeats there are a number of options:
//!
//! ### Fixed Number of Repeats
//!
//! The following specifies all bits in a fixed number of words:
//!```text
//! w[];n
//!```
//! Where `n` is the number of words.
//!
//! For instance to specify 48 complete words from word 3, use:
//!```text
//! 3[];48
//!```
//!
//! ### Dependent Number of Repeats
//!
//! The number of words is often given by a field that comes before the repeat. This can be specifed by:
//!```text
//! w[];(v[])⁑n
//!```
//!
//! Where `v` is the word containing the number of repeats, `⁑` is a condition and `n` is the limit. Conditions allowed are `<` (less then) and `<=` (less than or equal). Note that is highly recommanded that a limit is set so that any clients can set maximum buffer sizes.
//!
//! For instance, if word 2 contains the number of repeated words and this is followed by the repeated word up to a max of 48 then use:
//!```text
//! 3[];(2[])<49
//!```
//!
//!
//! Alternatively one could another condition to mean the same thing:
//!```text
//! 3[];(2[])<=48
//!```
//!
//! ### Variable Number of Repeats
//! The number of words is variable and only a limit is known.
//!
//! This can be specified as:
//!
//! ```text
//! w[];⁑n
//! ```
//! or, if a range of words repeats, as:
//!
//! ```text
//! w[]..v[];⁑n
//! ```
//! Where `v` is the word containing the number of repeats, `⁑` is a condition and `n` is the limit.
//! Conditions allowed are `<` (less then) and `<=` (less than or equal). Note that is highly recommanded that a limit is set so that any clients can set maximum buffer sizes.
//!
//! For instance a repeating list of u16 values starting at  word 3 and limited to 1024 values would be
//! represented as:
//!
//! ```text
//! 3[]..4[];<=1024
//! ```
//!
//! ### Literals
//! The actual state of the bits can be set using a literal. This can be shown with the following examples:
//! - Using hexadecimal to set word 0
//! ```text
//!  [0x23FF]
//! ```
//! - Using binary to set word 5
//! ```text
//! 5[0b1101_0001]
//! ```
//!
//! # Example Code
//! ```
//! use bit_lang::{BitRange, BitSpec,  Repeat, Word};
//!
//! fn main() {
//!     let data = "5[3..7]";
//!     let bit_spec = bit_lang::parse(data).unwrap();
//!   
//!     assert_eq!( bit_spec.start.index, 5);
//!     assert_eq!( bit_spec.start.bit_range, BitRange::Range(3,7));
//! }
//! ```

// TODO:
// - Provide a function that can be directly used with #[serde(deserialize_with = "??")]
// - The user needs to calculate the position of a variable word which can be complicated, espacially
//   if this is after a varaible repeatiing group. A posible solution would be to to assign
//   a symbolic name to a bit spec and then have a notation that the position of the word(group) is
//   after that. Prosposal example:
//       frequencies: 4[]..5[];(3[])<20
//       station_count: 0[];after frequencies
//       [0]..1[];(station_count)<10 after station_count
//   The positions are adapted based on what was before (this raises the question as to why
//   use abolution positions at all?).
//   If using something like ddgen then the symbolic names do not need to be specified, but
//   are the same as the field name..#[allow(dead_code)]
// TODO
// Update the docs to include the new variable repeats

pub mod bit_spec;
pub mod parser;

use std::fmt::Display;

pub use bit_spec::{BitRange, BitSpec, Repeat, Word};

#[derive(Debug, PartialEq, Clone)]
pub enum Error {
    ParseError,
    IllFormed,
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Error in bit specification")
    }
}

/// Parse the bit-lang specification and return a BitSpec.
pub fn parse(bit_spec_string: &str) -> Result<BitSpec, Error> {
    let (remaining, bit_spec) = parser::bit_spec(bit_spec_string).map_err(|_| Error::ParseError)?;

    // All characters in the bit spec shoudl have been consumed
    if remaining.is_empty() {
        Ok(bit_spec)
    } else {
        Err(Error::IllFormed)
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    #[test]
    fn test_bit_spec_with_simple_forms() {
        let data = "4";
        let bit_spec = parse(data).unwrap();
        let expected = BitSpec {
            start: Word {
                index: 0,
                bit_range: BitRange::Single(4),
            },
            end: None,
            repeat: Repeat::None,
        };
        assert_eq!(bit_spec, expected);

        let data = "4..6";
        let bit_spec = parse(data).unwrap();
        let expected = BitSpec {
            start: Word {
                index: 0,
                bit_range: BitRange::Range(4, 6),
            },
            end: None,
            repeat: Repeat::None,
        };
        assert_eq!(bit_spec, expected);

        let data = "[4..6]";
        let bit_spec = parse(data).unwrap();
        let expected = BitSpec {
            start: Word {
                index: 0,
                bit_range: BitRange::Range(4, 6),
            },
            end: None,
            repeat: Repeat::None,
        };
        assert_eq!(bit_spec, expected);

        let data = "5[3..7]";
        let bit_spec = parse(data).unwrap();
        let expected = BitSpec {
            start: Word {
                index: 5,
                bit_range: BitRange::Range(3, 7),
            },
            end: None,
            repeat: Repeat::None,
        };
        assert_eq!(bit_spec, expected);

        let data = "5[]";
        let bit_spec = parse(data).unwrap();
        let expected = BitSpec {
            start: Word {
                index: 5,
                bit_range: BitRange::WholeWord,
            },
            end: None,
            repeat: Repeat::None,
        };
        assert_eq!(bit_spec, expected);
    }

    #[test]
    fn test_bit_spec_with_fixed_repeat() {
        let data = "3[4..7]..6[0..5];48";

        let bit_spec = parse(data).unwrap();

        let expected = BitSpec {
            start: Word {
                index: 3,
                bit_range: BitRange::Range(4, 7),
            },
            end: Some(Word {
                index: 6,
                bit_range: BitRange::Range(0, 5),
            }),
            repeat: Repeat::Fixed { number: 48 },
        };
        assert_eq!(bit_spec, expected);
    }

    #[test]
    fn test_bit_spec_with_dependent_word_repeat() {
        let data = "4[]..7[];(3[])<49";
        let bit_spec = parse(data).unwrap();
        let expected_repeat = Repeat::Dependent {
            bit_spec: Box::new(BitSpec {
                start: Word {
                    index: 3,
                    bit_range: BitRange::WholeWord,
                },
                end: None,
                repeat: Repeat::None,
            }),
            limit: 48,
        };
        let expected = BitSpec {
            start: Word {
                index: 4,
                bit_range: BitRange::WholeWord,
            },
            end: Some(Word {
                index: 7,
                bit_range: BitRange::WholeWord,
            }),
            repeat: expected_repeat,
        };
        assert_eq!(bit_spec, expected);
    }

    #[test]
    fn test_bit_spec() {
        let data = "3[4..7]..6[0..5]";

        let bit_spec = parse(data).unwrap();

        let expected = BitSpec {
            start: Word {
                index: 3,
                bit_range: BitRange::Range(4, 7),
            },
            end: Some(Word {
                index: 6,
                bit_range: BitRange::Range(0, 5),
            }),
            repeat: Repeat::None,
        };
        assert_eq!(bit_spec, expected);

        let data = "4[]..7[]";
        let bit_spec = parse(data).unwrap();
        let expected = BitSpec {
            start: Word {
                index: 4,
                bit_range: BitRange::WholeWord,
            },
            end: Some(Word {
                index: 7,
                bit_range: BitRange::WholeWord,
            }),
            repeat: Repeat::None,
        };
        assert_eq!(bit_spec, expected);

        let data = "[]..5[]";
        let bit_spec = parse(data).unwrap();
        let expected = BitSpec {
            start: Word {
                index: 0,
                bit_range: BitRange::WholeWord,
            },
            end: Some(Word {
                index: 5,
                bit_range: BitRange::WholeWord,
            }),
            repeat: Repeat::None,
        };
        assert_eq!(bit_spec, expected);

        let data = "[]";
        let bit_spec = parse(data).unwrap();
        let expected = BitSpec {
            start: Word {
                index: 0,
                bit_range: BitRange::WholeWord,
            },
            end: None,
            repeat: Repeat::None,
        };
        assert_eq!(bit_spec, expected);

        let data = "[]..6[0..5]";
        let bit_spec = parse(data).unwrap();
        let expected = BitSpec {
            start: Word {
                index: 0,
                bit_range: BitRange::WholeWord,
            },
            end: Some(Word {
                index: 6,
                bit_range: BitRange::Range(0, 5),
            }),
            repeat: Repeat::None,
        };
        assert_eq!(bit_spec, expected);
    }

    #[test]
    fn test_max_size() {
        let bit_spec = parse("4[]").unwrap();
        assert_eq!(bit_spec.max_size(), 1);

        let bit_spec = parse("4[]..7[]").unwrap();
        assert_eq!(bit_spec.max_size(), 4);

        let bit_spec = parse("4[]..5[]").unwrap();
        assert_eq!(bit_spec.max_size(), 2);

        let bit_spec = parse("4[]..5[];5").unwrap();
        assert_eq!(bit_spec.max_size(), 10);

        let bit_spec = parse("4[]..5[];(3[])<6").unwrap();
        assert_eq!(bit_spec.max_size(), 10);

        let bit_spec = parse("4[]..5[];(3[])<=6").unwrap();
        assert_eq!(bit_spec.max_size(), 12);
    }

    #[test]
    fn test_literal() {
        let bit_spec = parse("4[0xBA]").unwrap();

        assert_eq!(bit_spec.start.index, 4);
        assert_eq!(
            bit_spec.start.bit_range,
            BitRange::Literal("0xBA".to_string())
        );
    }

    #[test]
    fn test_nonsense() {
        assert!(parse("abcd").is_err());
    }

    #[test]
    fn test_ill_formed() {
        // This incorrect bit spec should  give an error `"3[];(2[])10"`

        assert!(parse("3[];(2[])10").is_err());
    }
}
