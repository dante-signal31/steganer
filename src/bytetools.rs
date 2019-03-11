/// Module to perform byte operations.

/// Convert 3 bytes to a 24 bits long integer.
///
/// bytes[0] is shifted to most significant position, while bytes[1] is kept
/// at middle position and bytes[2] is left at least significant position.
///
/// As rust has no u24, what is returned is an u32 with its first byte set to 0.
pub fn bytes_to_u24(bytes: &[u8; 3])-> u32 {
    ((bytes[0] as u32) << 16) + ((bytes[1] as u32) << 8) + (bytes[2] as u32)
}

/// Convert a 24 bit long integer into an array of 3 bytes.
///
/// Most significant bits are stored at first byte while least significant
/// bits are left at last byte.
///
/// If given int is longer than 24 bits then exceeding bits are discarded.
pub fn u24_to_bytes(int: u32)-> [u8; 3]{
    let lower_byte = (int & mask(8, false)) as u8;
    let middle_byte = ((int >> 8) & mask(8, false)) as u8;
    let upper_byte = ((int >> 16) & mask(8, false)) as u8;
    [upper_byte, middle_byte, lower_byte]
}

/// Return a mask to apply to binary operations.
///
/// # Parameters
/// * length: Number of 1's from least significant bit. Every other bit is set to 0.
/// * inverted: If true then a number of 0's equal to length is placed from least significant bit.
/// Every other bit is set to 1.
///
/// # Returns
/// * A mask coded in an u32.
///
/// # Example
/// ```rust
/// use steganer::bytetools::mask;
///
/// let mask_normal = mask(3, false);
/// assert_eq!(mask_normal, 0b_0000_0111 as u32);
///
/// let mask_inverted = mask(3, true);
/// assert_eq!(mask_inverted, 0b_1111_1000 as u32);
/// ```
pub fn mask(length: u8, inverted: bool)-> u32 {
    let normal_mask: u32 = 2u32.pow(length as u32) - 1;
    match inverted {
        true=> !normal_mask,
        false=> normal_mask,
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
        let normal_mask = mask(3, false);
        let expected_normal_mask = 0b_0000_0111 as u32;
        assert_eq!(normal_mask, expected_normal_mask,
                   "Normal mask not properly generated. Expected mask was {:#b} but we've got {:#b}",
                   expected_normal_mask, normal_mask);
        let inverted_mask = mask(3, true);
        let expected_inverted_mask = 0b_11111111_11111111_11111111_1111_1000 as u32;
        assert_eq!(inverted_mask, expected_inverted_mask,
                   "Inverted mask not properly generated. Expected mask was {:#b} but we've got {:#b}",
                   expected_inverted_mask, inverted_mask);
    }



}