use serde::{de::Error, Deserialize, Deserializer};

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct BitRange(u8, u8);

#[derive(Debug, Clone)]
pub enum BitRangeError {
    InvalidRangeFormat,
    ReversedRange,
}

impl<'de> Deserialize<'de> for BitRange {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: String = Deserialize::deserialize(deserializer)?;
        if let Ok(n) = s.parse::<u8>() {
            return Ok(BitRange(n, n));
        }

        let mut parts = s.split("..");
        let start = parts
            .next()
            .ok_or("invalid range format")
            .map_err(D::Error::custom)?
            .parse::<u8>()
            .map_err(|_| D::Error::custom("invalid range"))?;

        let end = parts
            .next()
            .ok_or("invalid range format")
            .map_err(D::Error::custom)?
            .parse::<u8>()
            .map_err(|_| D::Error::custom("invalid range format"))?;

        if start <= end {
            Ok(BitRange(start, end))
        } else {
            Err(D::Error::custom("reversed range"))
        }
    }
}

impl BitRange {
    pub fn is_single_bit(&self) -> bool {
        self.0 == self.1
    }

    pub fn start(&self) -> u8 {
        self.0
    }

    pub fn end(&self) -> u8 {
        self.1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use serde::Deserialize;

    #[test]
    fn single_bound() {
        #[derive(Deserialize)]
        struct Bits {
            bit_range: BitRange,
        }

        let bits: Bits = toml::from_str(r#"bit_range = "2""#).unwrap();

        assert_eq!(bits.bit_range.start(), 2);
        assert_eq!(bits.bit_range.end(), 2);
    }

    #[test]
    fn range() {
        #[derive(Deserialize)]
        struct Bits {
            bit_range: BitRange,
        }

        let bits: Bits = toml::from_str(r#"bit_range = "2..5""#).unwrap();

        assert_eq!(bits.bit_range.start(), 2);
        assert_eq!(bits.bit_range.end(), 5);
    }

    #[test]
    fn range_descending_error() {
        #[derive(Deserialize)]
        struct Bits {
            bit_range: BitRange,
        }

        assert!(toml::from_str::<Bits>(r#"bit_range = "5..1""#).is_err());
    }

    #[test]
    fn single_bit_range() {
        #[derive(Deserialize)]
        struct Bits {
            bit_range: BitRange,
        }

        let bits: Bits = toml::from_str(r#"bit_range = "2..6""#).unwrap();

        assert_eq!(bits.bit_range.start(), 2);
        assert_eq!(bits.bit_range.end(), 6);
    }
}
