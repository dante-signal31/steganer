/// Module to read file to hide contents and to write extracted content to a destination file.
///
/// Thanks to ContentReader type you can get an iterator to read a file to hide and get its bits
/// in predefined bunches. Every bunch of bits are returned inside a Chunk type.
///
/// Conversely, FileWriter allows you write chunks of bits into a destination file.
///
/// # Usage example:
/// ```rust
/// use steganer::fileio::{FileContent, ContentReader, FileWriter};
///
/// let file_content = FileContent::new("source_file.txt")
///                         .expect("Error obtaining source file content");
/// let mut reader = ContentReader::new(&file_content, 4)
///                     .expect("There was a problem reading source file.");;
/// {
///     let mut writer = FileWriter::new("output_file")
///                     .expect("Error creating output file for extracted data.");
///     for chunk in reader {
///         // Do things with every chunk of 4 bits of data from source_file.txt.
///         writer.write(chunk);
///     }
/// } // When FileWriter types get out of scope they write to file pending last few bytes.
/// // At this point contents of source_file.txt and output_file.txt should be the same.
/// ```
use std::fmt;
use std::fmt::{Debug, Formatter};
use std::fs::File;
// Write import gets a compiler warning. It warns about importing Write is useless but actually
// if I remove Write import I get a compiler error in this module code.
use std::io::{BufReader, Read, Write, Error};
use std::iter::Iterator;
use std::ops::Add;
// Write import gets a compiler warning. It warns about importing PathBuf is useless but actually
// if I remove PathBuf import I get a compiler error in this module code.
use std::path::PathBuf;

use bitreader::{BitReader, BitReaderError};

use crate::bytetools::{u24_to_bytes, mask, bytes_to_u24, get_bits};


/// Bits read from files to be hidden are stored at Chunks.
pub struct Chunk {
    /// Every Chunk stores a maximum of 32 read bits at this property, those bits are
    /// at natural order (Big Endian) and justified to right.
    pub data: u32,
    /// Number of bits actually stored at data attribute. If you are reading the last few file bits
    /// you're probably going read less bits than requested.
    pub length: u8,
    /// An index about relative position of this chunk at file to be hidden.
    pub order: u32,
}

impl Chunk {
    #[must_use]
    pub fn new(data: u32, length: u8, order: u32)-> Self {
        Chunk {data, length, order}
    }
}

/// Type to represent excess bits that are not enough to conform an entire byte.
#[derive(PartialEq, Clone)]
struct Remainder {
    /// u8 with remainder data bits insufficient to conform a byte. Bits are right justified.
    data: u8,
    /// u8 with how many bits of remainder are actual data.
    length: u8,
}

impl Remainder {
    #[must_use]
    pub fn new(data: u8, length: u8)-> Self{
        Remainder {data, length}
    }
}

/// This Add is not a binary sum but a binary accumulator instead.
///
/// RHS bits are accumulated at the end of first operand bits. So this operation is not
/// commutative. Everything is left justified.
///
/// A BinaryAccumulationResult is returned after this operation.
///
/// This implementation was possible thanks to
/// [this Stackoverflow post](https://stackoverflow.com/questions/28005134/how-do-i-implement-the-add-trait-for-a-reference-to-a-struct)
impl<'a, 'b> Add<&'b Remainder> for &'a Remainder {
    type Output = BinaryAccumulation;

    fn add(self, rhs: &'b Remainder) -> Self::Output {
        let total_length = self.length + rhs.length;
        if total_length <= 8 {
            let shifted_bits_to_add = rhs.data << (8 - self.length - rhs.length);
            let accumulated_bits = self.data + shifted_bits_to_add;
            if total_length == 8 {
                BinaryAccumulation {
                    complete_byte: Some(accumulated_bits),
                    remainder: None,
                }
            } else {
                BinaryAccumulation {
                    complete_byte: None,
                    remainder: Some(Remainder::new(accumulated_bits, total_length)),
                }
            }
        } else {
            let shifted_left_hand_side_bits = (self.data as u16) << 8;
            let shifted_bits_to_add = (rhs.data as u16) << (16 - self.length - rhs.length);
            let accumulated_bits = shifted_left_hand_side_bits + shifted_bits_to_add;
            let remainder_length = total_length - 8;
            let complete_byte = ((accumulated_bits & (!mask(8, false) as u16)) >> 8) as u8;
            let remainder_bits = (accumulated_bits & (mask(8-remainder_length, true) as u16)) as u8;
            BinaryAccumulation {
                complete_byte: Some(complete_byte),
                remainder: Some(Remainder::new(remainder_bits, remainder_length)),
            }
        }
    }
}

impl Debug for Remainder {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "(data: {}, length: {})",
               self.data, self.length)
    }
}

/// Type returned after binary accumulate two remainders.
#[derive(PartialEq)]
struct BinaryAccumulation {
    complete_byte: Option<u8>,
    remainder: Option<Remainder>,
}

impl Debug for BinaryAccumulation {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let complete_byte = match &self.complete_byte {
            Some(value)=> *value,
            None=> 0,
        };
        let remainder = match &self.remainder {
            Some(remainder)=> (*remainder).clone(),
            None=> Remainder::new(0,0),
        };
        write!(f, "(complete_byte: {}, remainder: {:?})",
               complete_byte, remainder)
    }
}

/// Wrapper around file contents.
///
/// Once this type is created with its *new()* method file is automatically read and its contents
/// is placed at *self.content* attribute.
pub struct FileContent {
    /// File to be read.
    source: File,
    /// Vector of bytes with read content.
    content: Vec<u8>,
}

impl FileContent {
    #[must_use]
    pub fn new(source_file: &str)-> Result<Self, Error> {
        let source = File::open(source_file)?;
        let mut buf_reader = BufReader::new(&source);
        let mut content: Vec<u8> = Vec::new();
        let _ = buf_reader.read_to_end(&mut content)
            .expect("Error reading file to hide content.");
        Ok(FileContent {
            source,
            content,
        })
    }
}

/// ContentReader gives you an iterator to read a FileContent data.
///
/// Iterator returns a Chunk Type with bits read in every read iteration.
pub struct ContentReader<'a> {
    /// BitReader type to read bits alone.
    bit_reader: BitReader<'a>,
    /// Amount of bits to get in each iterator round.
    chunk_size: u8,
    /// Index about how many read rounds we've done using iterator.
    position: u32,
}

impl<'a> ContentReader<'a> {
    #[must_use]
    pub fn new(content: &'a FileContent, chunk_size: u8)-> Result<Self, Error> {
        let file_bytes = content.content.as_slice();
        Ok(ContentReader {
            bit_reader: BitReader::new(file_bytes.clone()),
            chunk_size,
            position: 0,
        })
    }
}

/// Iterator to read file content a chunk at a time.
///
/// Iterator will try to read *self.chunk_size* bits at a time. So returned chunk's length attribute
/// is going to be equal to *self.chunk_size* unless we are really near to the file end. In that
/// last case less than self.chunk_size will be actually read so chunk's length attribute will
/// have the actual number of bits that were actually read.
impl<'a> Iterator for ContentReader<'a> {
    type Item = Chunk;

    fn next(&mut self) -> Option<Self::Item> {
        let chunk = match self.bit_reader.read_u32(self.chunk_size) {
            Ok(bits)=> {
                self.position += 1;
                Some(Chunk::new(bits, self.chunk_size, self.position))
            }
            Err(e)=> {
                if let BitReaderError::NotEnoughData {position, length, requested: _ } = e {
                    let available_bits = length - position;
                    if available_bits > 0 {
                        let bits = self.bit_reader.read_u32(available_bits as u8)
                            .expect("Error reading last few bits from file to be hidden.");
                        self.position += 1;
                        Some(Chunk::new(bits, available_bits as u8, self.position))
                    } else {
                        None
                    }
                } else {
                    panic!("Error reading data to be hidden");
                }
            }
        };
        chunk
    }
}

/// Wrapper over an open file to write into it chunks extracted from host files.
///
/// Complete bytes are written at once but border bytes need to be rebuild from two different
/// chunks, so we need *self.pending_data* to use as a temporal container until it is filled
/// completely and we can write it.
pub struct FileWriter {
    /// Destination file to write chunks into.
    destination: File,
    /// Buffer to write into extracted bits until we have a complete byte to write into
    /// destination.
    pending_data: Option<Remainder>,
}

impl FileWriter {
    #[must_use]
    pub fn new(destination_file: &str)-> Result<Self, Error> {
        let destination = File::create(destination_file)?;
        let initial_remainder = None;
        Ok(FileWriter{destination, pending_data: initial_remainder})
    }

    /// Write Chunk into *self.destination* file.
    ///
    /// Actually only complete bytes will be written into file. Incomplete remainder bytes
    /// will be stored into self.pending_bytes until they fill up. When pending_bytes fills
    /// it is written and replaced by new exceeding bits.
    pub fn write(&mut self, chunk: Chunk)-> std::io::Result<()>{
        if let Some(complete_bytes) = self.store_remainder(&chunk){
            for byte in complete_bytes.iter(){
                let _ = self.destination.write(&[*byte])
                    .expect("An IO error happened when trying to write chunk to output file.");
            }
        }
        Ok(())
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
    /// ```rust
    /// use steganer::fileio::FileWriter;
    ///
    /// let data = 0b_11_u32;
    /// let returned_data = FileWriter::left_justify(data, 2);
    /// assert_eq!(0b_1100_0000_u8, returned_data[0]);
    /// ```
    pub fn left_justify(data: u32, data_length: u8)-> [u8; 3]{
        let left_shift = 24 - data_length; // Remember 8 leftmost bits are discarded.
        let justified_data = data << left_shift;
        u24_to_bytes(justified_data)
    }

    /// Get bits that do not conform complete bytes.
    ///
    /// # Parameters:
    /// * data: Chunk data already left justified.
    /// * length: How many bits from left are actual data.
    ///
    /// # Returns:
    /// * A Some(Remainder) if a remainder is available.
    /// * None is returned if there is no remainder available (i.e data conforms an integer
    /// amount of bytes).
    fn get_remainder(data: u32, length: u8)-> Option<Remainder>{
        let remainder_length = length % 8;
        if remainder_length == 0 {
            None
        } else {
            let complete_bytes = length / 8;
            let remainder_bits = get_bits(data, complete_bytes*8, remainder_length) as u8;
            let left_justified_remainder_bits = remainder_bits << (8 - remainder_length);
            Some(Remainder::new(left_justified_remainder_bits, remainder_length))
        }
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

    /// Called by *store_remainder()* to get a left justified u32 with current remainder with
    /// chunk data appended.
    ///
    /// # Parameters:
    /// * chunk: Chunk to append.
    ///
    /// # Returns:
    /// * u32 with left justified current remainder with chunk data appended.
    /// * u8 how many bits from left are actual data.
    fn append_to_remainder(&self, chunk: &Chunk)-> (u32, u8){
        let left_justified_data = Self::left_justify(chunk.data, chunk.length);
        let data_int = bytes_to_u24(&left_justified_data);
        let (pending_data, pending_data_length) = match &self.pending_data {
            Some(remainder)=> (remainder.data, remainder.length),
            None=> {
                let default_remainder = Remainder::new(0, 0);
                (default_remainder.data, default_remainder.length)
            }
        };
        let pending_data_left_justified = (pending_data as u32) << (32 - 8);
        let data_appended_to_remainder: u32 = pending_data_left_justified + (data_int << (8 - pending_data_length));
        let total_length = pending_data_length + chunk.length;
        (data_appended_to_remainder, total_length)
    }

    /// Keep in *self.pending_data* those bits that are not enough to conform a complete byte.
    ///
    /// Bits are accumulated until they fill a byte. If adding bits to *self.pending_data* fills
    /// entire bytes, then those bytes are returned in a vector and excess bits become the
    /// new *self.pending_data*.
    ///
    /// # Parameters:
    /// * chunk: Chunk to be written.
    ///
    /// # Returns:
    /// * Optionally returns a vector with complete bytes if adding remainder to *self.pending_data* fills
    /// any. If that does not happen a None is returned instead.
    fn store_remainder(&mut self, chunk: &Chunk)-> Option<Vec<u8>> {
        let (data_appended_to_remainder, total_length) = self.append_to_remainder(chunk);
        if let Some(new_remainder) = Self::get_remainder(data_appended_to_remainder, total_length){
            let non_remainder_length = total_length - new_remainder.length;
            self.pending_data = Some(new_remainder);
            if non_remainder_length == 0 {
                // Only remainder. No entire bytes.
                None
            } else {
                // Remainder and entire bytes.
                // I don't use get_bits() because I want to keep non_remainder_data left justified.
                let non_remainder_data = data_appended_to_remainder & mask(32-non_remainder_length, true);
                Some(Self::get_bytes(non_remainder_data, non_remainder_length)
                    .expect("Could not extract any byte from provided data"))
            }
        } else {
            // Only entire bytes. No remainder left.
            self.pending_data = None;
            Some(Self::get_bytes(data_appended_to_remainder, total_length)
                .expect("Could not extract any byte from provided data"))
        }
    }
}

impl Drop for FileWriter {
    /// On drop, self.pending_data content is considered complete and should be stored
    /// into self.destination.
    fn drop(&mut self) {
        if let Some(remainder) = &self.pending_data {
            let _ = self.destination.write(&[remainder.data])
                .expect("An IO error happened when trying to write chunk to output file.");;
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
//    use super::super::test_common::{TestEnvironment, hash_file};
    use std::path::Path;
    use std::io::{Cursor, Read};
    use byteorder::{NativeEndian, ReadBytesExt, WriteBytesExt};

    use test_common::{TestEnvironment, hash_file};

    const MESSAGE: &str = "Lorem ipsum dolor sit amet, consectetur adipiscing elit, \
    sed eiusmod tempor incidunt ut labore et dolore magna aliqua.";
    const SOURCE_FILE: &str = "source.txt";

    /// Called by get_temporaty_test_file() to include some dummy content into test file.
    fn populate_test_file(test_env: &TestEnvironment) -> PathBuf {
        let source_path = Path::new(test_env.path()).join(SOURCE_FILE);
        let mut source_file = File::create(&source_path)
            .expect("Could not create test source file");
        let file_content = String::from(MESSAGE);
        source_file.write_all(file_content.as_bytes())
            .expect("Error populating test source file.");
        source_path
    }

    /// Populate a test file in a temporary folder.
    ///
    /// # Returns:
    /// * PathBuf: Path to created temporary file. Includes folder path and file name.
    /// * TesEnvironment: Handle to temporary folder. Keep it in scope, if it leaves from scope then
    ///temporary folder  is removed.
    fn get_temporary_test_file()-> (PathBuf, TestEnvironment) {
        let test_env = TestEnvironment::new();
        let source_path = populate_test_file(&test_env);
        (source_path, test_env)
    }

    /// Convert a vector of 4 bytes to u32.
    ///
    /// It assumes those bytes are in Big Endian order (natural order).
    fn bytes_to_u32(bytes: Vec<u8>)-> u32 {
        assert_eq!(bytes.len(), 4);
        ((bytes[0] as u32) << 24) +
            ((bytes[1] as u32) << 16) +
            ((bytes[2] as u32) <<  8) +
            ((bytes[3] as u32) <<  0)

    }

    /// Justify to right the first "size" bits.
    ///
    /// Be aware that even chunks may have some zeroed bits at the very beginning, so we should
    /// shift less positions to justify to right.
    ///
    /// # Parameters:
    /// * data: Data chunk stored in an u32.
    /// * size: Actual size in bits of data chunk.
    /// * odd: True if this datachunk was read at an odd position.
    ///
    /// # Returns:
    /// * u32: Data chunk stored in an u32 were bits were justified at right.
    fn normalize(data: u32, size: u8, odd: bool)-> u32{
        let shift = 32 - size;
        let remainder = size % 8;
        if odd {
            data >> shift
        } else {
            data >> shift - remainder
        }

    }

    #[test]
    // Test iteration with chunks smaller than 8 bits.
    fn test_iterator_next_under_8() { ;
        let ( source_path,test_env) = get_temporary_test_file();
        let file_content = FileContent::new(source_path.to_str()
            .expect("Source file name contains odd characters."))
            .expect("Error getting file contents");
        let mut reader = ContentReader::new(&file_content, 4)
            .expect("There was a problem reading source file.");
        let mut chunk: Chunk = reader.next()
            .expect("Error reading chunk"); // Upper half of "L".
        let mut expected_chunk = "L".to_owned().as_bytes()[0] as u32;
        // Remove lower half of "L".
        expected_chunk = expected_chunk & 0xF0;
        expected_chunk = expected_chunk >> 4;
        assert_eq!(expected_chunk, chunk.data);
        reader.next(); // Lower half of "L".
        reader.next(); // Upper half of "o".
        chunk = reader.next()
            .expect("Error reading chunk"); // Lower half of "o".
        expected_chunk = "o".to_owned().as_bytes()[0] as u32;
        expected_chunk = expected_chunk & 0x0F;
        assert_eq!(expected_chunk, chunk.data);
    }

    #[test]
    // Test iteration with chunks bigger than 8 bits.
    fn test_iterator_next_over_8() {
        let ( source_path,test_env) = get_temporary_test_file();
        let file_content = FileContent::new(source_path.to_str()
            .expect("Source file name contains odd characters."))
            .expect("Error getting file contents");
        let mut reader = ContentReader::new(&file_content, 12)
            .expect("There was a problem reading source file.");
        let mut chunk = reader.next()
            .expect("Error reading chunk"); // "L" and upper half of "o".
        let mut expected_chunk_vec = "Lo".to_owned().into_bytes();
        // rdr = [0b0100_1100, 0b0110_1111, 0b0000_0000, 0b0000_0000] --> Lo
        let mut rdr = Cursor::new(vec!(expected_chunk_vec[0],
                                        expected_chunk_vec[1],
                                        0 as u8,
                                        0 as u8));
        // expected_chunk = 0b0110_1111_0100_1100 --> On Intel: Little-Endian: oL
        let mut expected_chunk= rdr.read_u32::<NativeEndian>()
            .expect("Error reading chunk bigger than 8");
        // expected_chunk = 0b0110_0000_0100_1100 --> We remove lower half of "o".
        expected_chunk = expected_chunk & 0xF0FF;
        let mut wtr: Vec<u8> = Vec::new();
        // wtr = [0100_1100, 0110_0000, 0, 0]
        wtr.write_u32::<NativeEndian>(expected_chunk)
            .expect("Error writing chunk bigger than 8.");
        let mut expected_int = normalize(bytes_to_u32(wtr), 12, true);
        // expected_int = 0b0100_1100_0110
        assert_eq!(expected_int, chunk.data);
        reader.next(); // Lower half of "o" and "r".
        reader.next(); // "e" and upper half of "m".
        chunk = reader.next()
            .expect("Error reading chunk"); // Lower half "m" and " " --> 0b1101_0010_0000
        // expected_chunk_vec = [0b0110_1101, 0b0010_0000]
        expected_chunk_vec = "m ".to_owned().into_bytes();
        rdr = Cursor::new(vec!(expected_chunk_vec[0],
                               expected_chunk_vec[1],
                               0 as u8,
                               0 as u8));
        // expected_chunk = 0b0010_0000_0110_1101
        expected_chunk= rdr.read_u32::<NativeEndian>()
            .expect("Error reading chunk bigger than 8");
        // expected_chunk = 0b0010_0000_0000_1101 --> Remove upper half of "m".
        expected_chunk = expected_chunk & 0xFF0F;
        wtr = Vec::new();
        // wtr = [0000_1101, 0010_0000, 0, 0]
        wtr.write_u32::<NativeEndian>(expected_chunk)
            .expect("Error writing chunk bigger than 8.");
        // expected_int = 0b1101_0010_0000
        expected_int = normalize(bytes_to_u32(wtr), 12, false);
        // chunk_data = 0b1101_0010_0000
        assert_eq!(expected_int, chunk.data);
    }

    fn test_writing_n_bits_chunks(chunk_size: u8) {
        // Source file creation.
        let ( source_path,test_env) = get_temporary_test_file();
        let file_content = FileContent::new(source_path.to_str()
            .expect("Source file name contains odd characters."))
            .expect("Error getting file contents");
        let mut reader = ContentReader::new(&file_content, chunk_size)
            .expect("There was a problem reading source file.");
        // Destination file setup.
        let destination_file_name_path = test_env.path().join("output.txt").into_os_string().into_string()
            .expect("Error reading destination file name. Unsupported character might have been used.");
        {
            // We enclose destination_writer in its own scope so drop() is called at that scope end
            // to write remaining bits to destination file.
            let mut destination_writer = FileWriter::new(destination_file_name_path.as_str())
                .expect("Error happened trying to created FileWriter type.");
            // Transferring chunks.
            for chunk in reader {
                destination_writer.write(chunk);
            }
        }
        // Test destination file has same content than source file.
        let source_file_hash = hash_file(source_path.to_str()
            .expect("Source file name contains odd characters"))
            .expect("Something wrong happened when calculating hash for source file.");
        let destination_file_hash = hash_file(destination_file_name_path.as_str())
            .expect("Something wrong happened when calculating hash for destination file.");
        assert_eq!(source_file_hash.as_ref(), destination_file_hash.as_ref(),
                   "Destination file content is not the same as source file content. \
                   Source hash is {:X?} and destination is {:X?}",
                   source_file_hash.as_ref(), destination_file_hash.as_ref());
    }

    #[test]
    fn test_writing_23_bits_chunks() {
        test_writing_n_bits_chunks(23);
    }

    #[test]
    fn test_writing_12_bits_chunks() {
        test_writing_n_bits_chunks(12);
    }

    #[test]
    fn test_writing_8_bits_chunks() {
        test_writing_n_bits_chunks(8);
    }

    #[test]
    fn test_writing_4_bits_chunks() {
        test_writing_n_bits_chunks(4);
    }

    #[test]
    fn test_writing_3_bits_chunks() {
        test_writing_n_bits_chunks(3);
    }

    #[test]
    fn test_left_justify() {
        let data = 0b_11_u32;
        let returned_data = FileWriter::left_justify(data, 2);
        assert_eq!(0b_1100_0000_u8, returned_data[0]);
    }

    #[test]
    fn test_get_remainder() {
        let expected_remainder = 0b_1011_0000_u8;
        let data_1_length = 4_u8;
        let data_1 = (expected_remainder as u32) << (32 - data_1_length - 4);
        let data_2_length = 12_u8;
        let data_2 = (expected_remainder as u32) << (32 - data_2_length - 4);
        let data_3_length = 20_u8;
        let data_3 = (expected_remainder as u32) << (32 - data_3_length - 4);
        let remainder1 = FileWriter::get_remainder(data_1, data_1_length)
            .expect("No remainder found");
        assert_eq!((expected_remainder, 4), (remainder1.data, remainder1.length),
                   "We did not get expected remainder when analyzing 1 byte case. Expected {:#?}, but got {:#?}.",
                   (expected_remainder, 4), (remainder1.data, remainder1.length));
        let remainder2 = FileWriter::get_remainder(data_2, data_2_length)
            .expect("No remainder found");
        assert_eq!((expected_remainder, 4), (remainder2.data, remainder2.length),
                   "We did not get expected remainder when analyzing 2 byte case. Expected {:#?}, but got {:#?}.",
                   (expected_remainder, 4), (remainder2.data, remainder2.length));
        let remainder3 = FileWriter::get_remainder(data_3, data_3_length)
            .expect("No remainder found");
        assert_eq!((expected_remainder, 4), (remainder3.data, remainder3.length),
                   "We did not get expected remainder when analyzing 3 bytes case. Expected {:#?}, but got {:#?}.",
                   (expected_remainder, 4), (remainder3.data, remainder3.length));
    }

    #[test]
    fn test_add_remainder() {
        // Accumulating without overflow.
        let remainder1 = Remainder::new(0b_101_0_0000_u8, 3);
        let remainder2 = Remainder::new(0b_11_u8, 2);
        let expected_result = BinaryAccumulation {
            complete_byte: None,
            remainder: Some(Remainder::new(0b_1011_1_000_u8, 5))
        };
        let result = &remainder1 + &remainder2;
        assert_eq!(expected_result, result,
                   "Accumulation without overflow did not worked as we expected. \
                   We expected a remainder of {:?} but we got {:?}",
                   expected_result, result);
        // Accumulating with overflow.
        let remainder1 = Remainder::new(0b_1010_111_0_u8, 7);
        let remainder2 = Remainder::new(0b_011_u8, 3);
        let expected_result = BinaryAccumulation {
            complete_byte: Some(0b_1010_1110_u8),
            remainder: Some(Remainder::new(0b_11_00_0000_u8, 2))
        };
        let result = &remainder1 + &remainder2;
        assert_eq!(expected_result, result,
                   "Accumulation with overflow did not worked as we expected. \
                   We expected a remainder of {:?} but we got {:?}",
                   expected_result, result);
        // Accumulating an exact byte.
        let remainder1 = Remainder::new(0b_1010_111_0_u8, 7);
        let remainder2 = Remainder::new(0b_0_u8, 1);
        let expected_result = BinaryAccumulation {
            complete_byte: Some(0b_1010_1110_u8),
            remainder: None,
        };
        let result = &remainder1 + &remainder2;
        assert_eq!(expected_result, result,
                   "Accumulation with overflow did not worked as we expected. \
                   We expected a remainder of {:?} but we got {:?}",
                   expected_result, result);
    }

    #[test]
    fn test_store_remainder() {
        // Destination file setup.
        let ( _,test_env) = get_temporary_test_file();
        let destination_file_name_path = test_env.path().join("output.txt").into_os_string().into_string()
            .expect("Error reading destination file name. Unsupported character might have been used.");
        {
            // We enclose destination_writer in its own scope so drop() is called at that scope end
            // to write remaining bits to destination file.
            let mut destination_writer = FileWriter::new(destination_file_name_path.as_str())
                .expect("Error happened trying to created FileWriter type.");
            // Accumulating without overflow.
            let remainder1 = Remainder::new(0b_101_0_0000_u8, 3);
            let remainder2 = Chunk::new(0b_11, 2, 1);
            let expected_result = 0b_1011_1_000_u8;
            destination_writer.pending_data = Some(remainder1);
            if let Some(complete_byte) = destination_writer.store_remainder(&remainder2) {
                assert!(false, "A complete byte was returned when no remainder fill was expected.");
            } else {
                if let Some(remainder) = &destination_writer.pending_data {
                    assert_eq!(expected_result, remainder.data,
                               "Store remainder without overflow did not worked as we expected. \
                            We expected a remainder of {:#b} but we got {:#b}",
                               expected_result, remainder.data);
                } else {
                    assert!(false, "We expected a remainder but none was found.");
                }
            }
            // Accumulating with overflow.
            let remainder1 = Remainder::new(0b_1010_111_0_u8, 7);
            let remainder2 = Chunk::new(0b_011, 3, 1);
            let expected_result = 0b_11_00_0000_u8;
            let expected_complete_byte = 0b_1010_1110_u8;
            destination_writer.pending_data = Some(remainder1);
            if let Some(complete_byte) = destination_writer.store_remainder(&remainder2){
                if let Some(remainder) = &destination_writer.pending_data {
                    assert_eq!(expected_result, remainder.data,
                               "Store remainder with overflow did not worked as we expected. \
                                We expected a remainder of {:#b} but we got {:#b}",
                               expected_result, remainder.data);
                    assert_eq!(expected_complete_byte, complete_byte[0],
                               "Recovered complete byte was not what we were expecting. \
                               We expected {:#b} but we got {:#b}",
                               expected_complete_byte, complete_byte[0]);
                } else {
                    assert!(false, "We expected a remainder but none was found.");
                }
            } else {
                assert!(false, "We were expecting to fill remainder but no complete byte was returned.");
            }
            // Accumulating an exact byte.
            let remainder1 = Remainder::new(0b_1010_111_0_u8, 7);
            let remainder2 = Chunk::new(0b_0, 1, 1);
            let expected_complete_byte = 0b_1010_1110_u8;
            destination_writer.pending_data = Some(remainder1);
            if let Some(complete_byte) = destination_writer.store_remainder(&remainder2){
                if let Some(remainder) = &destination_writer.pending_data {
                    assert!(false, "We expected no remainder but one found instead. Found remainder \
                        has data {:#b} a length {}",
                        remainder.data, remainder.length);
                } else {
                    assert_eq!(expected_complete_byte, complete_byte[0],
                               "Recovered complete byte was not what we were expecting. \
                               We expected {:#b} but we got {:#b}",
                               expected_complete_byte, complete_byte[0]);
                }
            } else {
                assert!(false, "We were expecting to fill remainder but no complete byte was returned.");
            }
        }
    }

    #[test]
    fn test_get_bytes() {
        // Not enough bits to fill a byte.
        let data_incomplete_byte_length = 5_u8;
        let data_incomplete_byte = (0b_1_0101 as u32) << (32 - data_incomplete_byte_length);
        if let None = FileWriter::get_bytes(data_incomplete_byte, data_incomplete_byte_length) {
            assert!(true);
        }
        // Enough bits to fill a byte and partially a second.
        let data_up_to_second_byte_length = 13_u8;
        let data_up_to_second_byte = (0b_1_0101 as u32) << (32 - data_up_to_second_byte_length);
        if let Some(bytes) = FileWriter::get_bytes(data_up_to_second_byte, data_up_to_second_byte_length) {
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
        if let Some(bytes) = FileWriter::get_bytes(data_up_to_third_byte, data_up_to_third_byte_length) {
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
        if let Some(bytes) = FileWriter::get_bytes(data_up_to_fourth_byte, data_up_to_fourth_byte_length) {
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

    #[test]
    fn test_append_to_remainder() {
        // Destination file setup.
        let ( _,test_env) = get_temporary_test_file();
        let destination_file_name_path = test_env.path().join("output.txt").into_os_string().into_string()
            .expect("Error reading destination file name. Unsupported character might have been used.");
        {
            // We enclose destination_writer in its own scope so drop() is called at that scope end
            // to write remaining bits to destination file.
            let mut destination_writer = FileWriter::new(destination_file_name_path.as_str())
                .expect("Error happened trying to created FileWriter type.");
            let current_remainder_length = 7_u8;
            let current_remainder = 0b_1010_1110_u8;
            destination_writer.pending_data = Some(Remainder::new(current_remainder, current_remainder_length));
            let chunk = Chunk::new(0b_011, 3, 1);
            let expected_appended_remainder_length = 10_u8;
            let expected_appended_remainder = 0b_1010_1110_11_u32 << 32 - expected_appended_remainder_length;
            let (appended_remainder, appended_remainder_length) = destination_writer.append_to_remainder(&chunk);
            assert_eq!(expected_appended_remainder, appended_remainder,
                       "Appended remainder is not what we were expecting. Expected {} but got {}.",
                       expected_appended_remainder, appended_remainder);
            assert_eq!(expected_appended_remainder_length, appended_remainder_length,
                       "Appended remainder length is not what we were expecting. Expected {} but got {}.",
                       expected_appended_remainder_length, appended_remainder_length);
        }
    }
}