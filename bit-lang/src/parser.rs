use crate::bit_spec::*;

use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::u8 as u8_parser,
    character::complete::{char, one_of},
    combinator::{map, opt, recognize, value},
    multi::{many0, many1},
    //number::complete::{i32, u8},
    sequence::{delimited, preceded, separated_pair, tuple},
    IResult,
};

// Parse function follow ...

fn index(input: &str) -> IResult<&str, u8> {
    (u8_parser)(input)
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
    alt((range, single_bit))(input) // Order important
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
            bit_range: BitRange::Literal(literal.to_string()),
        },
    ))
}

// word = bit_range | [index] "[" [bit_range] "]" | index "[" literal "]";   (* NEW *)
fn word(input: &str) -> IResult<&str, Word> {
    // Order of parsers is important as literal words could be initially mistaken for bit ranges
    let (remaining, word) = alt((literal_word, fully_qualified_word, bit_range_as_word))(input)?;

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

    let adjusted_limit = match condition {
        Condition::Lte => limit,
        Condition::Lt => limit - 1,
    };
    Ok((
        remaining,
        Repeat::Variable {
            word,
            limit: adjusted_limit.into(),
        },
    ))
}

// repeat = ";" (fixed_repeat  | variable_repeat)  ;
fn repeat(input: &str) -> IResult<&str, Repeat> {
    //let (remaining, (_, repeat)) = tuple((tag(";"), alt((variable_repeat, fixed_repeat))))(input)?;
    let (remaining, repeat) = preceded(tag(";"), alt((variable_repeat, fixed_repeat)))(input)?;

    Ok((remaining, repeat))
}

fn hexadecimal_literal(input: &str) -> IResult<&str, String> {
    let (input, _) = alt((tag("0x"), tag("0X")))(input)?;

    let (remaining, hex) = recognize(many1(one_of("0123456789abcdefABCDEF_")))(input)?;

    let literal = format!("0x{hex}");
    Ok((remaining, literal))
}

fn separater(input: &str) -> IResult<&str, char> {
    let (input, sep_char) = one_of("_")(input)?;

    Ok((input, sep_char))
}

fn boolean_char(input: &str) -> IResult<&str, char> {
    let (input, bool_char) = one_of("01")(input)?;
    Ok((input, bool_char))

    // TODO bin_digit
}

// <binary> ::= (<boolean_char> | <separator>)* <boolean_char> ( <boolean_char> | <separator>)*
fn binary(input: &str) -> IResult<&str, &str> {
    let (input, binary_number) = recognize(tuple((
        many1(boolean_char),
        many0(tuple((opt(separater), many1(boolean_char)))),
    )))(input)?;

    Ok((input, binary_number))
}

fn binary_literal(input: &str) -> IResult<&str, String> {
    let (input, _) = alt((tag("0b"), tag("0B")))(input)?;
    //let (remaining, bin) = recognize(many1(one_of("01_")))(input)?;
    //let (remaining, bin) = recognize(many1(one_of("01_")))(input)?;
    let (remaining, bin) = recognize(binary)(input)?;

    let literal = format!("0b{bin}");
    Ok((remaining, literal))
}

fn literal(input: &str) -> IResult<&str, String> {
    let (remaining, literal) = alt((hexadecimal_literal, binary_literal))(input)?;
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
    fn test_seperator() {
        let data = "_";
        let (_, c) = separater(data).unwrap();
        assert_eq!(c, '_');

        assert!(separater("1").is_err());
    }

    #[test]
    fn test_boolean_char() {
        let data = "1";
        let (_, c) = boolean_char(data).unwrap();
        assert_eq!(c, '1');

        let data = "0";
        let (_, c) = boolean_char(data).unwrap();
        assert_eq!(c, '0');

        let data = "_";
        assert!(boolean_char(data).is_err());
    }

    #[test]
    fn test_binary() {
        let data = "1";
        let (_, c) = binary(data).unwrap();
        assert_eq!(c, "1");

        let data = "1011001";
        let (_, c) = binary(data).unwrap();
        assert_eq!(c, "1011001");

        let data = "1011_001";
        let (_, c) = binary(data).unwrap();
        assert_eq!(c, "1011_001");
    }

    #[test]
    fn test_literal_word() {
        let data = "[0x1234]";
        let (_, r) = literal_word(data).unwrap();
        let expected = Word {
            index: 0,
            bit_range: BitRange::Literal("0x1234".to_string()),
        };
        assert_eq!(r, expected);

        let data = "4[0b0011_1100]";
        let (_, r) = literal_word(data).unwrap();
        let expected = Word {
            index: 4,
            bit_range: BitRange::Literal("0b0011_1100".to_string()),
        };
        assert_eq!(r, expected);

        let data = "5[0b0011ABCD]";
        assert!(literal_word(data).is_err());
    }

    #[test]
    fn test_bit_range_to_string() {
        assert_eq!(BitRange::Single(5).to_string(), "5");
        assert_eq!(BitRange::Single(5).to_string(), "5");
        assert_eq!(BitRange::WholeWord.to_string(), "");
        assert_eq!(BitRange::Literal("0x2E".to_string()).to_string(), "0x2E");
        assert_eq!(
            BitRange::Literal("0b1100".to_string()).to_string(),
            "0b1100"
        );
    }

    #[test]
    fn test_word_to_string() {
        assert_eq!(
            Word {
                bit_range: BitRange::Range(3, 6),
                index: 4
            }
            .to_string(),
            "4[3..6]".to_string()
        );
        assert_eq!(
            Word {
                bit_range: BitRange::WholeWord,
                index: 4
            }
            .to_string(),
            "4[]".to_string()
        );
        assert_eq!(
            Word {
                bit_range: BitRange::Single(5),
                index: 4
            }
            .to_string(),
            "4[5]".to_string()
        );
        assert_eq!(
            Word {
                bit_range: BitRange::Literal("0x340A".to_string()),
                index: 4
            }
            .to_string(),
            "4[0x340A]".to_string()
        );
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

        let expected = Repeat::Variable { word, limit: 48 };
        assert_eq!(r, expected);

        let data = ";(4[])";
        assert!(repeat(data).is_err());
    }

    #[test]
    fn test_repeat_to_string() {
        assert_eq!(Repeat::Fixed(12).to_string(), "12");

        assert_eq!(Repeat::None.to_string(), "");

        assert_eq!(
            Repeat::Variable {
                word: Word {
                    index: 4,
                    bit_range: BitRange::WholeWord,
                },
                limit: 48
            }
            .to_string(),
            "(4[])<=48"
        );
    }

    #[test]
    fn test_variable_repeat() {
        let data = "(4[])<=48";
        let (_, r) = variable_repeat(data).unwrap();
        let word = Word {
            index: 4,
            bit_range: BitRange::WholeWord,
        };

        let expected = Repeat::Variable { word, limit: 48 };
        assert_eq!(r, expected);

        let data = "(4[0..7])<49";
        let (_, r) = variable_repeat(data).unwrap();
        let word = Word {
            index: 4,
            bit_range: BitRange::Range(0, 7),
        };

        let expected = Repeat::Variable { word, limit: 48 };
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
        assert_eq!(r, "0xABCD".to_string());

        let data = "0b1011_1100";
        let (_, r) = literal(data).unwrap();
        assert_eq!(r, "0b1011_1100".to_string());

        let data = "0b_1011_11b0";
        assert!(literal(data).is_err());

        let data = "0Xzb";
        assert!(literal(data).is_err());
    }

    #[test]
    fn test_hexadecimal_literal() {
        let data = "0x45B7";
        let (_, hex) = hexadecimal_literal(data).unwrap();
        assert_eq!(hex, "0x45B7".to_string());

        let data = "0X45_B7";
        let (_, hex) = hexadecimal_literal(data).unwrap();
        assert_eq!(hex, "0x45_B7".to_string());

        let data = "45B7";
        assert!(hexadecimal_literal(data).is_err());

        let data = "0xZZ";
        assert!(hexadecimal_literal(data).is_err());

        let data = "0x45ZZ";
        let (_, hex) = hexadecimal_literal(data).unwrap();
        assert_eq!(hex, "0x45".to_string());
    }

    #[test]
    fn test_binary_literal() {
        let data = "0b10001100";
        let (_, bin) = binary_literal(data).unwrap();
        assert_eq!(bin, "0b10001100".to_string());

        let data = "0b1000_1100";
        let (_, bin) = binary_literal(data).unwrap();
        assert_eq!(bin, "0b1000_1100".to_string());

        let data = "0b100155";
        let (_, bin) = binary_literal(data).unwrap();
        assert_eq!(bin, "0b1001".to_string());

        let data = "0101";
        assert!(binary_literal(data).is_err())
    }
}
