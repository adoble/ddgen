use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::u8 as u8_parser,
    character::complete::{char, one_of},
    combinator::{map, opt, recognize, value},
    multi::many1,
    //number::complete::{i32, u8},
    sequence::{delimited, preceded, separated_pair, tuple},
    IResult,
};

#[derive(Debug, PartialEq, Clone)]
pub enum LiteralType {
    Hex(String),
    Bin(String),
}

#[derive(Debug, PartialEq, Clone)]
pub enum BitRange {
    Single(u8),
    Range(u8, u8),
    WholeWord,
    Literal(LiteralType),
}

//#[derive(Debug, PartialEq, Copy, Clone)]
#[derive(Debug, PartialEq, Clone)]
pub struct Word {
    // No index refers to index = 0
    pub index: usize,
    // No bit spec refers to the whole word
    pub bit_range: BitRange,
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Condition {
    Lt,
    Lte,
}

// #[derive(Debug, PartialEq, Copy, Clone)]
#[derive(Debug, PartialEq, Clone)]
pub enum Repeat {
    // A simple fixed number of repititions
    Fixed(usize),
    // A variable number of repitions determined by another word and limited
    Variable {
        word: Word,
        condition: Condition,
        limit: usize,
    },
    // No repeat has been specified.
    // Having this removes the need to have an extra Option
    None,
}

impl Repeat {
    // Get the max number of repeats specified.
    fn max_repeats(&self) -> usize {
        match self {
            Repeat::None => 1,
            Repeat::Fixed(number) => *number,
            Repeat::Variable {
                condition: Condition::Lt,
                limit,
                ..
            } => (*limit) - 1,
            Repeat::Variable {
                condition: Condition::Lte,
                limit,
                ..
            } => *limit,
        }
    }
}

// #[derive(Debug, PartialEq, Copy, Clone)]
#[derive(Debug, PartialEq, Clone)]
pub struct BitSpec {
    pub start: Word,
    pub end: Option<Word>,
    pub repeat: Repeat,
}

fn index(input: &str) -> IResult<&str, u8> {
    (u8_parser)(input)
}

impl BitSpec {
    /// Get the max size in bytes of an array that could
    /// contain the bit specification.
    pub fn max_size(&self) -> usize {
        let n_words = match self {
            BitSpec {
                start: _,
                end: None,
                ..
            } => 1,
            BitSpec {
                start: Word { index: w, .. },
                end: Some(Word { index: v, .. }),
                ..
            } => v - w + 1,
        };

        let repeats = self.repeat.max_repeats();

        n_words * repeats
    }
}

fn single_bit(input: &str) -> IResult<&str, BitRange> {
    let (remaining, position) = (index)(input)?;

    Ok((remaining, BitRange::Single(position)))
}

fn range(input: &str) -> IResult<&str, BitRange> {
    //tuple((index, tag(".."), index))(input)
    let (remaining, (start, stop)) = separated_pair(index, tag(".."), index)(input)?;

    Ok((remaining, BitRange::Range(start, stop)))
}

fn bit_range(input: &str) -> IResult<&str, BitRange> {
    alt((range, single_bit))(input) // Order importantre
}

fn fully_qualified_word(input: &str) -> IResult<&str, Word> {
    let (remaining, (index, _, bit_range, _)) =
        tuple((opt(index), tag("["), opt(bit_range), tag("]")))(input)?;

    let completed_bit_range = match bit_range {
        Some(bit_range) => bit_range,
        None => BitRange::WholeWord,
    };

    Ok((
        remaining,
        Word {
            index: index.unwrap_or(0).into(),
            bit_range: completed_bit_range,
        },
    ))
}

// A bit spec - e.g. "3" or "4..6"  is also treated as a full word, i.e.
// "0[3]" or "0[4,..6]" respectively. This function maps the bit spec to
// a word for later inclusion in highe level parsers
fn bit_range_as_word(input: &str) -> IResult<&str, Word> {
    let (remaining, bit_range) = (bit_range)(input)?;

    Ok((
        remaining,
        Word {
            index: 0,
            bit_range,
        },
    ))
}

fn literal_word(input: &str) -> IResult<&str, Word> {
    let (input, index) = opt(index)(input)?;
    let (input, _) = tag("[")(input)?;
    let (input, literal) = literal(input)?;
    let (remaining, _) = tag("]")(input)?;

    Ok((
        remaining,
        Word {
            index: index.unwrap_or(0).into(),
            bit_range: BitRange::Literal(literal),
        },
    ))
}

// word = bit_range | [index] "[" [bit_range] "]" | index "[" literal "]";   (* NEW *)
// TODO Ignore literals for the time being
fn word(input: &str) -> IResult<&str, Word> {
    let (remaining, word) = alt((fully_qualified_word, bit_range_as_word, literal_word))(input)?;

    Ok((remaining, word))
}

fn condition(input: &str) -> IResult<&str, Condition> {
    let (remaining, condition) = alt((
        value(Condition::Lte, tag("<=")),
        value(Condition::Lt, tag("<")),
    ))(input)?;

    Ok((remaining, condition))
}

fn fixed_repeat(input: &str) -> IResult<&str, Repeat> {
    //let (remaining, repeat) = map(u8_parser, |value| Repeat::Fixed(value))(input)?;
    //let (remaining, repeat) = map(u8_parser,  Repeat::Fixed)(input)?;
    let (remaining, repeat) = map(u8_parser, |r| Repeat::Fixed(r.into()))(input)?;

    Ok((remaining, repeat))
}

// variable_word = "(" word ")";
fn variable_word(input: &str) -> IResult<&str, Word> {
    // TODO see if  we can also use take_until() to solve ambiguity
    let (remaining, word) = delimited(char('('), word, char(')'))(input)?;
    Ok((remaining, word))
}

// variable_repeat = variable_word condition limit;
fn variable_repeat(input: &str) -> IResult<&str, Repeat> {
    let (remaining, (word, condition, limit)) =
        tuple((variable_word, condition, u8_parser))(input)?;
    Ok((
        remaining,
        Repeat::Variable {
            word,
            condition,
            limit: limit.into(),
        },
    ))
}

// repeat = ";" (fixed_repeat  | variable_repeat)  ;
fn repeat(input: &str) -> IResult<&str, Repeat> {
    //let (remaining, (_, repeat)) = tuple((tag(";"), alt((variable_repeat, fixed_repeat))))(input)?;
    let (remaining, repeat) = preceded(tag(";"), alt((variable_repeat, fixed_repeat)))(input)?;

    Ok((remaining, repeat))
}

fn hexadecimal(input: &str) -> IResult<&str, LiteralType> {
    // preceded(
    //     alt((tag("0x"), tag("0X"))),
    //     recognize(many1(terminated(
    //         one_of("0123456789abcdefABCDEF"),
    //         many0(char('_')),
    //     ))),
    // )
    // .parse(input)

    let (input, _) = alt((tag("0x"), tag("0X")))(input)?;
    //let (remaining, bin) = recognize(many1(one_of("01_")))(input)?;
    let (remaining, hex) =
        // recognize(all_consuming(many1(one_of("0123456789abcdefABCDEF_"))))(input)?;
        recognize(many1(one_of("0123456789abcdefABCDEF_")))(input)?;

    Ok((remaining, LiteralType::Hex(hex.to_string())))
}

fn binary(input: &str) -> IResult<&str, LiteralType> {
    let (input, _) = alt((tag("0b"), tag("0B")))(input)?;
    //let (remaining, bin) = recognize(many1(one_of("01_")))(input)?;
    let (remaining, bin) = recognize(many1(one_of("01_")))(input)?;

    Ok((remaining, LiteralType::Bin(bin.to_string())))
}

fn literal(input: &str) -> IResult<&str, LiteralType> {
    let (remaining, literal) = alt((hexadecimal, binary))(input)?;
    Ok((remaining, literal))
}

// This is the top level parser
// word_range = word [".." word] [repeat]
pub fn bit_spec(input: &str) -> IResult<&str, BitSpec> {
    let (remaining, (start, end, repeat)) =
        //tuple((word, opt(preceded(tag(".."), word)), opt(repeat)))(input)?;
        tuple((word, opt(preceded(tag(".."), word)), 
               map(opt(repeat), |r| r.unwrap_or(Repeat::None))))(input)?;

    Ok((remaining, BitSpec { start, end, repeat }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_literal_word() {
        let data = "[0x1234]";
        let (_, r) = literal_word(data).unwrap();
        let expected = Word {
            index: 0,
            bit_range: BitRange::Literal(LiteralType::Hex("1234".to_string())),
        };
        assert_eq!(r, expected);

        let data = "4[0b0011_1100]";
        let (_, r) = literal_word(data).unwrap();
        let expected = Word {
            index: 4,
            bit_range: BitRange::Literal(LiteralType::Bin("0011_1100".to_string())),
        };
        assert_eq!(r, expected);

        let data = "5[0b0011ABCD]";
        assert!(literal_word(data).is_err());
    }

    #[test]
    fn test_repeat() {
        let data = ";12";
        let (_, r) = repeat(data).unwrap();
        assert_eq!(r, Repeat::Fixed(12));

        let data = ";6";
        let (_, r) = repeat(data).unwrap();
        assert_eq!(r, Repeat::Fixed(6));

        let data = ";(4[])<49";
        let (_, r) = repeat(data).unwrap();
        let word = Word {
            index: 4,
            bit_range: BitRange::WholeWord,
        };

        let expected = Repeat::Variable {
            word,
            condition: Condition::Lt,
            limit: 49,
        };
        assert_eq!(r, expected);

        let data = ";(4[])";
        assert!(repeat(data).is_err());
    }

    #[test]
    fn test_variable_repeat() {
        let data = "(4[])<=48";
        let (_, r) = variable_repeat(data).unwrap();
        let word = Word {
            index: 4,
            bit_range: BitRange::WholeWord,
        };

        let expected = Repeat::Variable {
            word,
            condition: Condition::Lte,
            limit: 48,
        };
        assert_eq!(r, expected);

        let data = "(4[0..7])<49";
        let (_, r) = variable_repeat(data).unwrap();
        let word = Word {
            index: 4,
            bit_range: BitRange::Range(0, 7),
        };

        let expected = Repeat::Variable {
            word,
            condition: Condition::Lt,
            limit: 49,
        };
        assert_eq!(r, expected);
    }
    #[test]
    fn test_fixed_repeat() {
        let data = "48";
        let (_, r) = fixed_repeat(data).unwrap();
        assert_eq!(r, Repeat::Fixed(48));
    }

    #[test]
    fn test_condition() {
        let data = "<";
        let (_, r) = condition(data).unwrap();
        assert_eq!(r, Condition::Lt);

        let data = "<=";
        let (_, r) = condition(data).unwrap();
        assert_eq!(r, Condition::Lte);

        let data = "";
        assert!(condition(data).is_err());

        let data = "==";
        assert!(condition(data).is_err());
    }

    #[test]
    fn test_max_repeats() {
        let repeat = Repeat::None;
        assert_eq!(repeat.max_repeats(), 1);

        let repeat = Repeat::Fixed(5);
        assert_eq!(repeat.max_repeats(), 5);

        let repeat = Repeat::Variable {
            condition: Condition::Lt,
            limit: 6,
            word: Word {
                index: 5,
                bit_range: BitRange::WholeWord,
            },
        };
        assert_eq!(repeat.max_repeats(), 5);

        let repeat = Repeat::Variable {
            condition: Condition::Lte,
            limit: 6,
            word: Word {
                index: 5,
                bit_range: BitRange::WholeWord,
            },
        };
        assert_eq!(repeat.max_repeats(), 6);
    }

    #[test]
    fn test_bit_spec_with_simple_forms() {
        let data = "4";
        let (_, r) = bit_spec(data).unwrap();
        let expected = BitSpec {
            start: Word {
                index: 0,
                bit_range: BitRange::Single(4),
            },
            end: None,
            repeat: Repeat::None,
        };
        assert_eq!(r, expected);

        let data = "4..6";
        let (_, r) = bit_spec(data).unwrap();
        let expected = BitSpec {
            start: Word {
                index: 0,
                bit_range: BitRange::Range(4, 6),
            },
            end: None,
            repeat: Repeat::None,
        };
        assert_eq!(r, expected);

        let data = "[4..6]";
        let (_, r) = bit_spec(data).unwrap();
        let expected = BitSpec {
            start: Word {
                index: 0,
                bit_range: BitRange::Range(4, 6),
            },
            end: None,
            repeat: Repeat::None,
        };
        assert_eq!(r, expected);

        let data = "5[3..7]";
        let (_, r) = bit_spec(data).unwrap();
        let expected = BitSpec {
            start: Word {
                index: 5,
                bit_range: BitRange::Range(3, 7),
            },
            end: None,
            repeat: Repeat::None,
        };
        assert_eq!(r, expected);

        let data = "5[]";
        let (_, r) = bit_spec(data).unwrap();
        let expected = BitSpec {
            start: Word {
                index: 5,
                bit_range: BitRange::WholeWord,
            },
            end: None,
            repeat: Repeat::None,
        };
        assert_eq!(r, expected);
    }

    #[test]
    fn test_bit_spec_with_repeat() {
        let data = "3[4..7]..6[0..5];48";

        let (_, r) = bit_spec(data).unwrap();

        let expected = BitSpec {
            start: Word {
                index: 3,
                bit_range: BitRange::Range(4, 7),
            },
            end: Some(Word {
                index: 6,
                bit_range: BitRange::Range(0, 5),
            }),
            repeat: Repeat::Fixed(48),
        };
        assert_eq!(r, expected);

        let data = "4[]..7[];(3[])<49";
        let (_, r) = bit_spec(data).unwrap();
        let repeat = Repeat::Variable {
            word: Word {
                index: 3,
                bit_range: BitRange::WholeWord,
            },
            condition: Condition::Lt,
            limit: 49,
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
            repeat: repeat,
        };
        assert_eq!(r, expected);
    }
    #[test]
    fn test_bit_spec() {
        let data = "3[4..7]..6[0..5]";

        let (_, r) = bit_spec(data).unwrap();

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
        assert_eq!(r, expected);

        let data = "4[]..7[]";
        let (_, r) = bit_spec(data).unwrap();
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
        assert_eq!(r, expected);

        let data = "[]..5[]";
        let (_, r) = bit_spec(data).unwrap();
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
        assert_eq!(r, expected);

        let data = "[]";
        let (_, r) = bit_spec(data).unwrap();
        let expected = BitSpec {
            start: Word {
                index: 0,
                bit_range: BitRange::WholeWord,
            },
            end: None,
            repeat: Repeat::None,
        };
        assert_eq!(r, expected);

        let data = "[]..6[0..5]";
        let (_, r) = bit_spec(data).unwrap();
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
        assert_eq!(r, expected);
    }

    // Not recommanded, but still accepted patterns.
    #[test]
    fn test_bit_spec_special_cases() {
        let data = "3..5..4[]";
        let (_, r) = bit_spec(data).unwrap();
        let expected = BitSpec {
            start: Word {
                index: 0,
                bit_range: BitRange::Range(3, 5),
            },
            end: Some(Word {
                index: 4,
                bit_range: BitRange::WholeWord,
            }),
            repeat: Repeat::None,
        };
        assert_eq!(r, expected);
    }

    #[test]
    fn test_word() {
        let data = "3[2..6]";
        let (_, r) = word(data).unwrap();
        assert_eq!(
            r,
            Word {
                index: 3,
                bit_range: BitRange::Range(2, 6)
            }
        );

        let data = "4[]";
        let (_, r) = word(data).unwrap();
        assert_eq!(
            r,
            Word {
                index: 4,
                bit_range: BitRange::WholeWord
            }
        );

        let data = "[]";
        let (_, r) = word(data).unwrap();
        assert_eq!(
            r,
            Word {
                index: 0,
                bit_range: BitRange::WholeWord
            }
        );

        let data = "3[]";
        let (_, r) = word(data).unwrap();
        assert_eq!(
            r,
            Word {
                index: 3,
                bit_range: BitRange::WholeWord
            }
        );

        let data = "7";
        let (_, r) = word(data).unwrap();
        assert_eq!(
            r,
            Word {
                index: 0,
                bit_range: BitRange::Single(7)
            }
        )
    }

    #[test]
    fn test_full_word() {
        let data = "3[4..6]";
        let (_, r) = fully_qualified_word(data).unwrap();

        let expected_word = Word {
            index: 3,
            bit_range: BitRange::Range(4, 6),
        };

        assert_eq!(r, expected_word);

        let data = "9[]";
        let (_, r) = fully_qualified_word(data).unwrap();
        let expected_word = Word {
            index: 9,
            bit_range: BitRange::WholeWord,
        };
        assert_eq!(r, expected_word);

        let data = "[3..7]";
        let (_, r) = fully_qualified_word(data).unwrap();
        let expected_word = Word {
            index: 0,
            bit_range: BitRange::Range(3, 7),
        };
        assert_eq!(r, expected_word);

        let data = "[]";
        let (_, r) = fully_qualified_word(data).unwrap();
        let expected_word = Word {
            index: 0,
            bit_range: BitRange::WholeWord,
        };
        assert_eq!(r, expected_word);
    }

    #[test]
    fn test_bit_range() {
        let data = "4..6";
        let (_, r) = bit_range(data).unwrap();
        assert_eq!(r, BitRange::Range(4, 6));

        let data = "7";
        let (_, r) = bit_range(data).unwrap();
        assert_eq!(r, BitRange::Single(7));
    }

    #[test]
    fn test_single_bit() {
        let data = "2";
        let (_, r) = single_bit(data).unwrap();
        assert_eq!(r, BitRange::Single(2));
    }

    #[test]
    fn test_range() {
        let data = "2..45";
        let (_, r) = range(data).unwrap();
        assert_eq!(r, BitRange::Range(2, 45));
    }

    #[test]
    fn test_index() {
        let data = "34";
        let (_, i) = index(data).unwrap();
        assert_eq!(i, 34);

        let data = "7";
        let (_, i) = index(data).unwrap();
        assert_eq!(i, 7);

        let data = "48";
        let (_, i) = index(data).unwrap();
        assert_eq!(i, 48);
    }

    #[test]
    fn test_literal() {
        let data = "0xABCD";
        let (_, r) = literal(data).unwrap();
        assert_eq!(r, LiteralType::Hex("ABCD".to_string()));

        let data = "0b1011_1100";
        let (_, r) = literal(data).unwrap();
        assert_eq!(r, LiteralType::Bin("1011_1100".to_string()));

        // let data = "0b1011_11b0";
        // assert!(literal(data).is_err());

        // let data = "0Xab_zc";
        // assert!(literal(data).is_err());
    }

    #[test]
    fn test_hexadecimal() {
        let data = "0x45B7";
        let (_, hex) = hexadecimal(data).unwrap();
        assert_eq!(hex, LiteralType::Hex("45B7".to_string()));

        let data = "0X45_B7";
        let (_, hex) = hexadecimal(data).unwrap();
        assert_eq!(hex, LiteralType::Hex("45_B7".to_string()));
    }

    #[test]
    fn test_binary() {
        let data = "0b10001100";
        let (_, bin) = binary(data).unwrap();
        assert_eq!(bin, LiteralType::Bin("10001100".to_string()));

        let data = "0b1000_1100";
        let (_, bin) = binary(data).unwrap();
        assert_eq!(bin, LiteralType::Bin("1000_1100".to_string()));
    }
}
