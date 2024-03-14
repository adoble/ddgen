mod response;

#[cfg(test)]
mod tests {

    use std::io::Repeat;

    use super::*;

    use crate::response::ResponseWord;
    use bit_lang::{BitRange, BitSpec, Repeat as WordRepeat, Word};

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

        let r = data[w].bit(n);

        assert_eq!(r, true);
    }

    #[test]
    fn deserialize_field() {
        let spec = bit_lang::parse("3[4..6]").unwrap();

        let data = [
            0b0000_0000, // 0
            0b0000_0000, // 1
            0b0000_0000, // 2
            0b0101_0000, // 3   3[4..6] = 0d5
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

        let r = data[w].field(n, m);

        assert_eq!(r, 5);
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

        let r = u16::from_le_bytes([data[v], data[w].field(m, n)]);

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

        let r = u32::from_le_bytes([data[v], data[v + 1], data[v + 2], data[w].field(m, n)]);

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

        let mut d: [u8; 5] = [0; 5]; // 5 is repeat
        d.copy_from_slice(&data[w..(w + r)]);

        assert_eq!(d, expected);
    }
}
