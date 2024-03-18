pub trait RequestWord {
    fn new(bits: u8) -> Self;
    fn bits(&self) -> u8;

    // TODO this should be private?
    fn set_bits(&mut self, bits: u8) -> &mut Self;

    // Default implementations
    fn modify_bit(&mut self, position: u8, state: bool) -> &mut Self {
        let mut mask: u8 = 1 << position;

        let modified_bits = if state {
            // setting the bit
            self.bits() | mask
        } else {
            // clear the bit{
            mask = !mask;
            self.bits() & mask
        };

        self.set_bits(modified_bits);

        self
    }

    /// Modify the field as specified by the start and end bit positions.
    ///
    /// Warning: Attempting to modify the whole word or having end less then start
    /// will cause the function to panic!
    fn modify_field(&mut self, value: u8, start: u8, end: u8) -> &mut Self {
        let mask = ((1 << (end - start + 1)) - 1) << start;
        let cleared_bits = self.bits() & !mask;
        let new_bits = value << start;
        self.set_bits(cleared_bits | new_bits);

        self
    }
}

impl RequestWord for u8 {
    fn new(bits: u8) -> Self {
        bits
    }

    fn bits(&self) -> u8 {
        *self
    }

    fn set_bits(&mut self, bits: u8) -> &mut Self {
        *self = bits;
        self
    }
    // fn word(&self) -> &u8 {
    //     self
    // }
}
