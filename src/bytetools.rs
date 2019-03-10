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
pub fn u24_to_bytes(int: u32)-> [u8; 3]{
    [0, 0, 0]
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


}