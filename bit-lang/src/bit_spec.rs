use std::fmt;

// #[derive(Debug, PartialEq, Eq, Clone, Hash, PartialOrd, Ord)]
// pub enum LiteralType {
//     Hex(String),
//     Bin(String),
// }

// impl fmt::Display for LiteralType {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         match self {
//             LiteralType::Bin(literal) => write!(f, "0b{literal}"),
//             LiteralType::Hex(literal) => write!(f, "0x{literal}"),
//         }
//     }
// }

#[derive(Debug, PartialEq, Eq, Clone, Hash, PartialOrd, Ord)]
pub enum BitRange {
    Single(u8),
    Range(u8, u8),
    WholeWord,
    Literal(String),
}

impl fmt::Display for BitRange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BitRange::Single(position) => write!(f, "{position}"),
            BitRange::Range(start, end) => write!(f, "{start}..{end}"),
            BitRange::WholeWord => write!(f, ""),
            BitRange::Literal(literal) => write!(f, "{literal}"),
        }
    }
}

//#[derive(Debug, PartialEq, Copy, Clone)]
#[derive(Debug, PartialEq, Eq, Clone, Hash, PartialOrd, Ord)]
pub struct Word {
    // No index refers to index = 0
    pub index: usize,
    // No bit spec refers to the whole word
    pub bit_range: BitRange,
}

impl fmt::Display for Word {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}[{}]", self.index, self.bit_range)
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Condition {
    Lt,
    Lte,
}

// #[derive(Debug, PartialEq, Copy, Clone)]
#[derive(Debug, PartialEq, Eq, Clone, Hash, PartialOrd, Ord)]
pub enum Repeat {
    // A simple fixed number of repetitions
    Fixed(usize),
    // A variable number of repetations determined by another word and limited
    Variable { word: Word, limit: usize },
    // No repeat has been specified.
    // Having this removes the need to have an extra Option
    None,
}

impl Repeat {
    // Get the max number of repeats specified.
    pub fn max_repeats(&self) -> usize {
        match self {
            Repeat::None => 1,
            Repeat::Fixed(number) => *number,
            Repeat::Variable { limit, .. } => *limit,
        }
    }
}

impl fmt::Display for Repeat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Repeat::None => write!(f, ""),
            Repeat::Fixed(repeat) => write!(f, "{repeat}"),
            Repeat::Variable { word, limit } => write!(f, "({word})<={limit}"),
        }
    }
}
#[derive(Debug, Eq, PartialEq, Clone)]
pub enum WordRange {
    Fixed(usize, usize),
    Variable(usize, usize, Word),
}

// #[derive(Debug, PartialEq, Copy, Clone)]
#[derive(Debug, Eq, PartialEq, Clone, Hash, PartialOrd, Ord)]
pub struct BitSpec {
    /// The word at the start of a word range. If a
    /// a single word is specified then this is the
    /// only entry.  
    pub start: Word,
    /// The last word in a word range.
    pub end: Option<Word>,
    /// How the word is repeated, if at all.
    pub repeat: Repeat,
}

impl BitSpec {
    /// From a simple word, create a complete bit spec.
    pub fn from_word(word: &Word) -> BitSpec {
        BitSpec {
            start: word.clone(),
            end: None,
            repeat: Repeat::None,
        }
    }

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

    /// Determine how many bytes this bit spec would need. It returns a tuple
    /// with the fixed size of the element and an optional Word showing
    /// where the multiplier is located.
    pub fn size(&self) -> (usize, Option<Word>) {
        match self {
            BitSpec {
                start: _,
                end: None,
                repeat: Repeat::None,
            } => (1, None),
            BitSpec {
                start,
                end: Some(end),
                repeat: Repeat::None,
            } => (end.index - start.index + 1, None),
            BitSpec {
                start: _,
                end: None,
                repeat: Repeat::Fixed(repeats),
            } => (*repeats, None),
            BitSpec {
                start,
                end: Some(end),
                repeat: Repeat::Fixed(repeats),
            } => ((end.index - start.index + 1) * repeats, None),
            BitSpec {
                start: _,
                end: None,
                repeat: Repeat::Variable { word, .. },
            } => (1, Some(word.clone())),
            BitSpec {
                start,
                end: Some(end),
                repeat: Repeat::Variable { word, .. },
            } => (end.index - start.index + 1, Some(word.clone())),
        }
    }

    pub fn word_range(&self) -> WordRange {
        let start = self.start.index;

        let end = self.end.as_ref().map_or(start, |end_word| end_word.index);

        match &self.repeat {
            Repeat::Fixed(repeats) => {
                WordRange::Fixed(start, start + ((end - start + 1) * repeats) - 1)
            }
            Repeat::Variable { word, .. } => WordRange::Variable(start, end, word.clone()),
            Repeat::None => WordRange::Fixed(start, end),
        }
    }

    /// Suggest a type for a variable to hold the value
    /// specified by the bit spec.

    pub fn suggested_word_type(&self) -> String {
        // First cover the special case of a literal. Handling this
        // first reduces the compexisty of the match statements.
        match self {
            BitSpec {
                start:
                    Word {
                        bit_range: BitRange::Literal(_),
                        ..
                    },
                ..
            } => return "u8".to_string(),
            _ => (),
        };

        // Now cover the rest of the cases
        match self {
            BitSpec {
                start:
                    Word {
                        bit_range: BitRange::Single(_),
                        ..
                    },
                end: None,
                ..
            } => "bool",
            BitSpec {
                start: Word { .. },
                end: None,
                ..
            } => "u8",
            BitSpec {
                start: Word {
                    index: start_index, ..
                },
                end: Some(Word {
                    index: end_index, ..
                }),
                ..
            } => match end_index - start_index + 1 {
                2 => "u16",
                4 => "u32",
                8 => "u64",
                16 => "u128",
                _ => "usize",
            },
        }
        .to_string()
    }
}

impl fmt::Display for BitSpec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let BitSpec { start, end, repeat } = self;

        let s = format!("{start}");
        let e = match end {
            Some(word) => format!("..{word}"),
            None => String::new(),
        };
        let r = match repeat {
            Repeat::None => String::new(),
            _ => format!(";{repeat}"),
        };
        write!(f, "{s}{e}{r}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::parse;

    #[test]
    fn test_from_word() {
        let word = Word {
            index: 5,
            bit_range: BitRange::Range(5, 7),
        };

        let expected = BitSpec {
            start: Word {
                index: 5,
                bit_range: BitRange::Range(5, 7),
            },
            end: None,
            repeat: Repeat::None,
        };

        let bit_spec = BitSpec::from_word(&word);

        assert_eq!(bit_spec, expected);
    }

    // Checking how PartialOrd works for BitRange
    #[test]
    fn bit_range_ordering() {
        let lower = BitRange::Single(3);
        let upper = BitRange::Single(4);
        assert!(upper > lower);

        let lower = BitRange::Range(2, 5);
        let upper = BitRange::Range(5, 7);
        assert!(upper > lower);

        let lower = BitRange::Range(2, 5);
        let upper = BitRange::Range(4, 7);
        assert!(upper > lower);

        let lower = BitRange::WholeWord;
        let upper = BitRange::WholeWord;
        assert!(upper == lower);

        let lower = BitRange::Literal("0x02".to_string());
        let upper = BitRange::Literal("0xF5".to_string());
        assert!(upper > lower);
    }

    // Checking how PartialOrd works for Word
    #[test]
    fn simple_word_ordering() {
        let lower = Word {
            index: 1,
            bit_range: BitRange::WholeWord,
        };

        let upper = Word {
            index: 2,
            bit_range: BitRange::WholeWord,
        };

        assert!(upper > lower);
    }

    #[test]
    fn bit_spec_ordering() {
        let lower_bit_spec = BitSpec {
            start: Word {
                index: 1,
                bit_range: BitRange::WholeWord,
            },
            end: None,
            repeat: Repeat::None,
        };

        let upper_bit_spec = BitSpec {
            start: Word {
                index: 2,
                bit_range: BitRange::WholeWord,
            },
            end: None,
            repeat: Repeat::None,
        };

        assert!(upper_bit_spec > lower_bit_spec);
    }

    #[test]
    fn bit_spec_with_ranges_ordering() {
        let lower_bit_spec = BitSpec {
            start: Word {
                index: 1,
                bit_range: BitRange::WholeWord,
            },
            end: Some(Word {
                index: 3,
                bit_range: BitRange::WholeWord,
            }),
            repeat: Repeat::None,
        };

        let upper_bit_spec = BitSpec {
            start: Word {
                index: 4,
                bit_range: BitRange::WholeWord,
            },
            end: Some(Word {
                index: 5,
                bit_range: BitRange::WholeWord,
            }),
            repeat: Repeat::None,
        };

        assert!(upper_bit_spec > lower_bit_spec);
    }

    #[test]
    fn bit_spec_with_bit_ordering() {
        let lower_bit_spec = BitSpec {
            start: Word {
                index: 1,
                bit_range: BitRange::Single(4),
            },
            end: None,
            repeat: Repeat::None,
        };

        let upper_bit_spec = BitSpec {
            start: Word {
                index: 1,
                bit_range: BitRange::Single(5),
            },
            end: None,
            repeat: Repeat::None,
        };

        assert!(upper_bit_spec > lower_bit_spec);
    }

    #[test]
    fn ordering_bit_ranges() {
        let lower_bit_spec = BitSpec {
            start: Word {
                index: 1,
                bit_range: BitRange::Range(2, 5),
            },
            end: None,
            repeat: Repeat::None,
        };

        let upper_bit_spec = BitSpec {
            start: Word {
                index: 1,
                bit_range: BitRange::Range(6, 7),
            },
            end: None,
            repeat: Repeat::None,
        };

        assert!(upper_bit_spec > lower_bit_spec);
    }

    #[test]
    fn test_max_repeats() {
        let repeat = Repeat::None;
        assert_eq!(repeat.max_repeats(), 1);

        let repeat = Repeat::Fixed(5);
        assert_eq!(repeat.max_repeats(), 5);

        let repeat = Repeat::Variable {
            limit: 5,
            word: Word {
                index: 5,
                bit_range: BitRange::WholeWord,
            },
        };
        assert_eq!(repeat.max_repeats(), 5);

        let repeat = Repeat::Variable {
            limit: 6,
            word: Word {
                index: 5,
                bit_range: BitRange::WholeWord,
            },
        };
        assert_eq!(repeat.max_repeats(), 6);
    }

    #[test]
    fn test_bit_spec_to_string() {
        assert_eq!(
            BitSpec {
                start: Word {
                    index: 6,
                    bit_range: BitRange::Single(4)
                },
                end: None,
                repeat: Repeat::None
            }
            .to_string(),
            "6[4]"
        );

        assert_eq!(
            BitSpec {
                start: Word {
                    index: 6,
                    bit_range: BitRange::Single(4)
                },
                end: Some(Word {
                    index: 8,
                    bit_range: BitRange::Range(1, 6)
                }),
                repeat: Repeat::None
            }
            .to_string(),
            "6[4]..8[1..6]"
        );

        assert_eq!(
            BitSpec {
                start: Word {
                    index: 6,
                    bit_range: BitRange::WholeWord
                },
                end: Some(Word {
                    index: 8,
                    bit_range: BitRange::Range(1, 6)
                }),
                repeat: Repeat::Fixed(10)
            }
            .to_string(),
            "6[]..8[1..6];10"
        );

        assert_eq!(
            BitSpec {
                start: Word {
                    index: 6,
                    bit_range: BitRange::WholeWord
                },
                end: Some(Word {
                    index: 8,
                    bit_range: BitRange::WholeWord
                }),
                repeat: Repeat::Variable {
                    limit: 10,
                    word: Word {
                        index: 3,
                        bit_range: BitRange::WholeWord
                    }
                }
            }
            .to_string(),
            "6[]..8[];(3[])<=10"
        );
    }

    #[test]
    fn test_suggested_word_type() {
        let bit_spec = BitSpec {
            start: Word {
                index: 4,
                bit_range: BitRange::Single(5),
            },
            end: None,
            repeat: Repeat::None,
        };

        assert_eq!(bit_spec.suggested_word_type(), "bool");

        let bit_spec = BitSpec {
            start: Word {
                index: 4,
                bit_range: BitRange::Range(5, 7),
            },
            end: None,
            repeat: Repeat::None,
        };
        assert_eq!(bit_spec.suggested_word_type(), "u8");

        let bit_spec = BitSpec {
            start: Word {
                index: 4,
                bit_range: BitRange::WholeWord,
            },
            end: None,
            repeat: Repeat::None,
        };
        assert_eq!(bit_spec.suggested_word_type(), "u8");

        let bit_spec = BitSpec {
            start: Word {
                index: 4,
                bit_range: BitRange::WholeWord,
            },
            end: Some(Word {
                index: 5,
                bit_range: BitRange::WholeWord,
            }),
            repeat: Repeat::None,
        };
        assert_eq!(bit_spec.suggested_word_type(), "u16");

        let bit_spec = BitSpec {
            start: Word {
                index: 4,
                bit_range: BitRange::WholeWord,
            },
            end: Some(Word {
                index: 11,
                bit_range: BitRange::WholeWord,
            }),
            repeat: Repeat::None,
        };
        assert_eq!(bit_spec.suggested_word_type(), "u64");

        let bit_spec = BitSpec {
            start: Word {
                index: 4,
                bit_range: BitRange::Literal("0xB9".to_string()),
            },
            end: None,
            repeat: Repeat::None,
        };
        assert_eq!(bit_spec.suggested_word_type(), "u8");

        //todo!("more tests");
    }

    #[test]
    fn test_size_with_bit() {
        let bit_spec = parse("3").unwrap();
        let size = bit_spec.size();

        assert_eq!(size, (1, None));
    }

    #[test]
    fn test_size_with_word() {
        let bit_spec = parse("3[]").unwrap();
        let size = bit_spec.size();
        assert_eq!(size, (1, None));
    }

    #[test]
    fn test_size_with_word_range() {
        let bit_spec = parse("3[]..5[]").unwrap();
        let size = bit_spec.size();
        assert_eq!(size, (3, None));
    }

    #[test]
    fn test_size_with_fixed_repeating_word() {
        let bit_spec = parse("3[];4").unwrap();
        let size = bit_spec.size();
        assert_eq!(size, (4, None));
    }

    #[test]
    fn test_size_with_variable_repeating_word() {
        let bit_spec = parse("3[];(1[])<=4").unwrap();
        let size = bit_spec.size();
        assert_eq!(
            size,
            (
                1,
                Some(Word {
                    index: 1,
                    bit_range: BitRange::WholeWord,
                }),
            )
        );
    }

    #[test]
    fn test_size_with_variable_repeating_word_range() {
        let bit_spec = parse("3[]..6[];(1[])<=4").unwrap();
        let size = bit_spec.size();
        assert_eq!(
            size,
            (
                4,
                Some(Word {
                    index: 1,
                    bit_range: BitRange::WholeWord,
                }),
            )
        );
    }

    #[test]
    fn test_range_simple() {
        let bit_spec = parse("3[]").unwrap();

        let WordRange::Fixed(start, end) = bit_spec.word_range() else {
            panic!("Word range is variable")
        };
        assert!(start == 3);
        assert!(end == 3);

        let bit_spec = parse("4[]..6[]").unwrap();
        let WordRange::Fixed(start, end) = bit_spec.word_range() else {
            panic!("Word range is variable")
        };
        assert!(start == 4);
        assert!(end == 6);
    }

    #[test]
    fn test_range_repeating() {
        let bit_spec = parse("4[];5").unwrap();
        let WordRange::Fixed(start, end) = bit_spec.word_range() else {
            panic!("Word range is variable")
        };

        assert_eq!(start, 4);
        assert_eq!(end, 8);

        let bit_spec = parse("4[]..5[];5").unwrap();
        let WordRange::Fixed(start, end) = bit_spec.word_range() else {
            panic!("Word range is variable")
        };

        assert_eq!(start, 4);
        assert_eq!(end, 13);

        let bit_spec = parse("0[];6").unwrap();
        let WordRange::Fixed(start, end) = bit_spec.word_range() else {
            panic!("Word range is variable")
        };

        assert_eq!(start, 0);
        assert_eq!(end, 5);

        let bit_spec = parse("0[];1").unwrap();
        let WordRange::Fixed(start, end) = bit_spec.word_range() else {
            panic!("Word range is variable")
        };

        assert_eq!(start, 0);
        assert_eq!(end, 0);
    }

    #[test]
    fn test_range_variable() {
        let bit_spec = parse("6[];(5[])<4").unwrap();

        let WordRange::Variable(start, end, repeat_word) = bit_spec.word_range() else {
            panic!("Word range is not variable")
        };

        assert_eq!(start, 6);
        assert_eq!(end, 6);
        assert_eq!(
            repeat_word,
            Word {
                index: 5,
                bit_range: BitRange::WholeWord
            }
        );
    }
}
