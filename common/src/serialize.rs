/// LEN is size of the buffer array returned.
pub trait Serialize {
    /// Serializes a structure.
    /// It returns a tuple containing:
    /// - The number of bytes need to represent the fixed size members and
    ///   any repeating members that either have a fixed size or a dependent
    ///   member that specifies their size. The number of bytes is to show
    ///   what is actually valid.
    /// - An array containing the serialized bytes for the above. Note that
    ///   the array can be larger then the actual number of serialized bytes.
    ///   The number of bytes is to show what is actually valid.   
    /// - An  iterator that can be used to loop though the rest of
    ///   the serialized data if a variable repeat is used in the bit spec.
    ///   If no variable repeat was used then this is empty .
    fn serialize<const LEN: usize>(&self) -> (usize, [u8; LEN], impl Iterator<Item = u8>);
}
