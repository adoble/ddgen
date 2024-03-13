mod response;

#[cfg(test)]
mod tests {

    use super::*;

    use crate::response::ResponseWord;
    use bit_lang::{BitRange, BitSpec};

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

        let w: usize = spec.start.index.into();
        let br = spec.start.bit_range;

        let n = if let BitRange::Single(n) = br {
            n
        } else {
            assert!(false, "Single bit expected, other variant found");
            0
        };

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

        let w: usize = spec.start.index.into();
        let (n, m) = if let BitRange::Range(n, m) = spec.start.bit_range {
            (n, m)
        } else {
            assert!(false, "Range expected, other variant found");
            (0, 0)
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

        let v = spec.start.index;

        let w = if let Some(word) = spec.end {
            match word.bit_range {
                BitRange::WholeWord => word.index,
                _ => {
                    assert!(false, "Unexpected variant");
                    0
                }
            }
        } else {
            assert!(false, "End word not specified");
            0
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
            0b1111_0101, // 4  big end. Top four bits shoudl be ignored
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
}
