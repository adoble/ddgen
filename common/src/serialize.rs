/// LEN is size of the buffer array returned.
pub trait Serialize {
    /// Returns a tuple of the number of bytes and an array with the
    /// serialized bytes in. Note that the array can be larger
    /// then the actual number of serialized bytes. The number of
    /// bytes is to show what is actually valid.   
    fn serialize<const LEN: usize>(&self) -> (usize, [u8; LEN]);
}

pub trait SerializeVariable {
    /// For structures that contain a varaible number of elements that
    /// are not constrained by a dependency this function shoudl be
    /// used to serialize them.
    /// It returns a tuple containing:
    /// - The number of bytes need to represent the fixed size members and
    ///   any repeating members that either have a fixed size or a dependent
    ///   member that specifies their size. The number of bytes is to show
    ///   what is actually valid.
    /// - An array containing the serialized bytes for the above. Note that
    ///   the array can be larger then the actual number of serialized bytes.
    /// - An optional iterator that can be used to loop though the rest of the data.
    fn serialize<const LEN: usize>(&self) -> (usize, [u8; LEN], Option<impl Iterator<Item = u8>>);
}
