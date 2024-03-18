mod deserialize;
mod error;
mod request;
mod response;
mod serialize;

pub use crate::error::DeviceError;

// TODO extend with u16, u32 etc.
pub fn repeating_words_u8<const LEN: usize>(
    source: &[u8],
    start: usize,
    repeats: usize,
) -> [u8; LEN] {
    let mut buf: [u8; LEN] = [0; LEN];
    let mut i = 0;

    for b in &source[start..(start + repeats)] {
        buf[i] = *b;
        i += 1;
    }

    buf
}

// TODO extend with u16, u32 etc.
pub fn repeating_words_u16<const LEN: usize>(
    source: &[u8],
    start: usize,
    repeats: usize,
) -> [u16; LEN] {
    let mut buf: [u16; LEN] = [0; LEN];
    let mut i = 0;

    for b in source[start..].chunks(2).take(repeats) {
        buf[i] = u16::from_le_bytes([b[0], b[1]]);
        i += 1;
    }

    buf
}

// pub fn modify_bit(word: u8, position: u8, state: bool) -> u8 {
//     let mut mask: u8 = 1 << position;

//     let modifed_word = if state {
//         // setting the bit
//         word | mask
//     } else {
//         // clear the bit{
//         mask = !mask;
//         word & mask
//     };

//     modifed_word
// }

// /// Modify the field as specified by the start and end bit positions.
// ///
// /// Warning: Attempting to modify the whole word or having end less then start
// /// will cause the function to panic!
// pub fn modify_field(word: u8, value: u8, start: u8, end: u8) -> u8 {
//     let mask = ((1 << (end - start + 1)) - 1) << start;
//     let cleared_bits = word & !mask;
//     let new_bits = value << start;
//     cleared_bits | new_bits
// }

#[cfg(test)]
//mod test_bit_spec_impl;
mod tests;
