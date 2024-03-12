// TODO make the word size generic

pub trait ResponseWord {
    fn word(&self) -> &u8;

    /// Get a bit as bool at a particular position
    fn bit(&self, position: u8) -> bool {
        let mask: u8 = 1 << position;
        (self.word() & mask) > 0
    }

    // Get the field at the specified position
    fn field(&self, start: u8, end: u8) -> u8 {
        let mut mask: u8 = 0;

        for count in start..=end {
            let b = 1 << count;
            mask |= b;
        }
        let v = self.word() & mask;
        v >> start
    }
}

impl ResponseWord for u8 {
    fn word(&self) -> &u8 {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn response_bits() {
        let r: u8 = 0b0011_0011;

        assert_eq!(true, r.bit(1));
        assert_eq!(false, r.bit(6));
        assert_eq!(true, r.bit(4));
    }

    #[test]
    fn response_fields() {
        let r: u8 = 0b0011_0011;

        assert_eq!(r.field(0, 3), 3);
        assert_eq!(r.field(1, 4), 9);
    }
}
