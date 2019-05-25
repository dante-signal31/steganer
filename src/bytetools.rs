/// Module to perform byte operations.
use std::mem::size_of;
use num::Integer;
use std::ops::{BitAnd, Shl, Shr, BitOr, Not};
use std::fmt::Debug;

/// Convert 3 bytes to a 24 bits long integer.
///
/// bytes[0] is shifted to most significant position, while bytes[1] is kept
/// at middle position and bytes[2] is left at least significant position.
///
/// # Parameters:
/// * bytes: Reference to an array of 3 bytes.
///
/// # Returns:
/// * As rust has no u24, what is returned is an u32 with its first byte set to 0.
pub fn bytes_to_u24(bytes: &[u8; 3])-> u32 {
    ((bytes[0] as u32) << 16) + ((bytes[1] as u32) << 8) + (bytes[2] as u32)
}

/// Convert a 24 bit long integer into an array of 3 bytes.
///
/// Most significant bits are stored at first byte while least significant
/// bits are left at last byte.
///
/// If given int is longer than 24 bits then exceeding bits are discarded.
///
/// # Parameters:
/// * int: u32 to be split in bytes. As we split only 3 bytes bits from 25 position to 32
/// are discarded.
///
/// # Returns:
/// * Array of 3 bytes.
pub fn u24_to_bytes(int: u32)-> [u8; 3]{
    let lower_byte = (int & mask::<u32>(8, false)) as u8;
    let middle_byte = ((int >> 8) & mask::<u32>(8, false)) as u8;
    let upper_byte = ((int >> 16) & mask::<u32>(8, false)) as u8;
    [upper_byte, middle_byte, lower_byte]
}

/// Return a mask to apply to binary operations.
///
/// # Parameters:
/// * length: Number of 1's from least significant bit. Every other bit is set to 0.
/// * inverted: If true then a number of 0's equal to length is placed from least significant bit.
/// Every other bit is set to 1.
///
/// # Returns:
/// * A mask coded in the same type that generic parameter.
///
/// # Example:
/// ```
/// use steganer::bytetools::mask;
///
/// let mask_normal = mask::<u8>(3, false);
/// assert_eq!(mask_normal, 0b_0000_0111 as u8);
///
/// let mask_inverted = mask::<u32>(3, true);
/// assert_eq!(mask_inverted, 0b_1111_1111_1111_1111_1111_1111_1111_1000 as u32);
/// ```
pub fn mask<T>(length: u8, inverted: bool)-> T
    where
        T: Integer + std::convert::From<u8> + Shl<usize, Output=T> + BitOr<Output=T> + Not<Output=T> + Debug,
{
    let mut normal_mask = T::from(0_u8);
    for _ in 0..length {
        normal_mask = (normal_mask << 1 as usize) | T::from(1_u8);
    }
    match inverted {
        true=> !normal_mask,
        false=> normal_mask,
    }
}

/// Get bits from a given position.
///
/// # Parameters
/// * source: Type with data to get bits from. Works with every unsigned type from u128 to below.
/// * position: Zero indexed position from left to get bits from.
/// * length: Number o bits to get rightwards from position.
///
/// # Returns:
/// * Requested bits into a the same type as source.
///
/// # Example:
/// ```
/// use steganer::bytetools::get_bits;
///
/// let INT: u32 = 0b_0000_0000_0110_1001_0101_1100_1110_0011_u32;
/// let bits_u32 = get_bits(INT, 24,2);
/// assert_eq!(bits_u32, 0b_11u32);
/// ```
pub fn get_bits<T>(source: T, position: u8, length: u8)-> T
    where
        T: Integer + From<u8> + Shl<usize, Output=T> + Shr<usize, Output=T> +
        BitAnd<Output=T> + BitOr<Output=T> + Not<Output=T> + Debug
{
    let right_drift = (size_of::<T>() * 8) - (position as usize + length as usize);
    let bit_mask = mask::<T>(length, false) << right_drift;
    let extracted_bits = (source & bit_mask) >> right_drift;
    extracted_bits

}

/// Justify at top left given data.
///
/// Leftmost 8 bits are discarded, because although an u32 is entered an u24 is returned
/// distributed in 3 bytes.
///
/// # Parameters:
/// * data: u32 containing data.
/// * data_length: How many bits are actually useful data.
///
/// # Returns:
/// * An array of three bytes. Returned u24 leftmost bits are returned in first byte.
///
/// # Example:
/// ```
/// use steganer::bytetools::left_justify;
///
/// let data = 0b_11_u32;
/// let returned_data = left_justify(data, 2);
/// assert_eq!(0b_1100_0000_u8, returned_data[0]);
/// ```
pub fn left_justify(data: u32, data_length: u8)-> [u8; 3]{
    let left_shift = 24 - data_length; // Remember 8 leftmost bits are discarded.
    let justified_data = data << left_shift;
    u24_to_bytes(justified_data)
}

/// Take data bits and return a vector with its bytes.
///
/// # Parameters:
/// * data: Left justified u32 with data bits.
/// * length: How many bits from left are actual data.
///
/// # Returns:
/// * Vector with bytes extracted from data.
pub fn get_bytes(data: u32, length: u8)-> Option<Vec<u8>>{
    let complete_bytes = length / 8;
    let bytes_to_return = if length % 8 > 0 {complete_bytes + 1} else {complete_bytes};
    let mut returned_complete_bytes: Vec<u8> = Vec::new();
    if bytes_to_return > 0 {
        for i in 0..bytes_to_return{
            let extracted_byte = get_bits(data, i*8, 8) as u8;
            returned_complete_bytes.extend_from_slice(&[extracted_byte]);
        }
        Some(returned_complete_bytes)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const BYTES: [u8; 3] = [0b_0110_1001, 0b_0101_1100, 0b_1110_0011];
    const INT: u32 = 6905059;

    #[test]
    fn test_bytes_to_u24() {
        let returned_int = bytes_to_u24(&BYTES);
        assert_eq!(INT, returned_int,
                   "Bytes has not been correctly converted. Expected int was {} what we've got is {}",
                   INT, returned_int);
    }

    #[test]
    fn test_u24_to_bytes() {
        let returned_bytes = u24_to_bytes(INT);
        assert_eq!(BYTES, returned_bytes,
                   "Integer has not been correctly converted. Expected bytes where {:?} what we've got is {:?}",
                   BYTES, returned_bytes);
    }

    #[test]
    fn test_mask() {
        let normal_mask = mask::<u32>(3, false);
        let expected_normal_mask = 0b_0000_0111 as u32;
        assert_eq!(normal_mask, expected_normal_mask,
                   "Normal mask not properly generated. Expected mask was {:#b} but we've got {:#b}",
                   expected_normal_mask, normal_mask);
        let inverted_mask = mask::<u32>(3, true);
        let expected_inverted_mask = 0b_11111111_11111111_11111111_1111_1000 as u32;
        assert_eq!(inverted_mask, expected_inverted_mask,
                   "Inverted mask not properly generated. Expected mask was {:#b} but we've got {:#b}",
                   expected_inverted_mask, inverted_mask);
    }

    #[test]
    fn test_get_bits() {
        let bits_u32 = get_bits(INT, 24,2) as u32;
        assert_eq!(bits_u32, 0b_11u32,
                   "Bits not properly extracted from u32. Expected {:#b} but we've got {:#b}",
                   0b_11u32, bits_u32);
        let bits_u64 = get_bits(INT as u64, 48, 4) as u64;
        assert_eq!(bits_u64, 0b_0101u64,
                   "Bits not properly extracted from u64. Expected {:#b} but we've got {:#b}",
                   0b_0101u64, bits_u64);
        let bits_u8 = get_bits(0b_0001_1000_u8, 3, 2 ) as u8;
        assert_eq!(bits_u8, 0b_11_u8,
                   "Bits not properly extracted. Expected {:#b} but we've got {:#b}",
                   0b_11_u8, bits_u8);
    }

    #[test]
    fn test_left_justify() {
        let data = 0b_11_u32;
        let returned_data = left_justify(data, 2);
        assert_eq!(0b_1100_0000_u8, returned_data[0]);
    }

    #[test]
    fn test_get_bytes() {
        // Not enough bits to fill a byte.
        let data_incomplete_byte_length = 5_u8;
        let data_incomplete_byte = (0b_1_0101 as u32) << (32 - data_incomplete_byte_length);
        if let None = get_bytes(data_incomplete_byte, data_incomplete_byte_length) {
            assert!(true);
        }
        // Enough bits to fill a byte and partially a second.
        let data_up_to_second_byte_length = 13_u8;
        let data_up_to_second_byte = (0b_1_0101 as u32) << (32 - data_up_to_second_byte_length);
        if let Some(bytes) = get_bytes(data_up_to_second_byte, data_up_to_second_byte_length) {
            assert_eq!(0_u8, bytes[0],
                       "Recovered first byte was not what we were expecting. Expected {} but got {}.",
                       0_u8, bytes[0]);
            assert_eq!(0b_1010_1000_u8, bytes[1],
                       "Recovered second byte was not what we were expecting. Expected {} but got {}.",
                       0b_1010_1000_u8, bytes[1]);
        }
        // Enough bits to fill two bytes and partially a third.
        let data_up_to_third_byte_length = 21_u8;
        let data_up_to_third_byte = (0b_1_0101 as u32) << (32 - data_up_to_third_byte_length);
        if let Some(bytes) = get_bytes(data_up_to_third_byte, data_up_to_third_byte_length) {
            assert_eq!(0_u8, bytes[0],
                       "Recovered first byte was not what we were expecting. Expected {} but got {}.",
                       0_u8, bytes[0]);
            assert_eq!(0_u8, bytes[1],
                       "Recovered second byte was not what we were expecting. Expected {} but got {}.",
                       0_u8, bytes[1]);
            assert_eq!(0b_1010_1000_u8, bytes[2],
                       "Recovered third byte was not what we were expecting. Expected {} but got {}.",
                       0b_1010_1000_u8, bytes[2]);
        }
        // Enough bits to fill three bytes and partially a fourth.
        let data_up_to_fourth_byte_length = 29_u8;
        let data_up_to_fourth_byte = (0b_1_0101 as u32) << (32 - data_up_to_fourth_byte_length);
        if let Some(bytes) = get_bytes(data_up_to_fourth_byte, data_up_to_fourth_byte_length) {
            assert_eq!(0_u8, bytes[0],
                       "Recovered first byte was not what we were expecting. Expected {} but got {}.",
                       0_u8, bytes[0]);
            assert_eq!(0_u8, bytes[1],
                       "Recovered second byte was not what we were expecting. Expected {} but got {}.",
                       0_u8, bytes[1]);
            assert_eq!(0_u8, bytes[2],
                       "Recovered third byte was not what we were expecting. Expected {} but got {}.",
                       0_u8, bytes[2]);
            assert_eq!(0b_1010_1000_u8, bytes[3],
                       "Recovered fourth byte was not what we were expecting. Expected {} but got {}.",
                       0b_1010_1000_u8, bytes[3]);
        }
    }
}