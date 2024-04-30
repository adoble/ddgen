/// LEN is size of the buffer array returned.
pub trait Serialize {
    /// Returns a tuple of the number of bytes and an array with the
    /// serialized bytes in. Note that the array can be larger
    /// then the actual number of serialized bytes. The number of
    /// bytes is to show what is actually valid.   
    fn serialize<const LEN: usize>(&self) -> (usize, [u8; LEN]);
}
