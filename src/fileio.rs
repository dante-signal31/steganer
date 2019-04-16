/// Module to read file to hide contents.
///
/// Thanks to ContentReader type you can get an iterator to read a file to hide and get its bits
/// in predefined bunches. Every bunch of bits are returned inside a Chunk type.
///
/// # Usage example:
/// ```rust
/// use steganer::fileio::{FileContent, ContentReader};
///
/// let file_content = FileContent::new("source_file.txt");
/// let reader = ContentReader::new(&file_content, 4);
/// for chunk in reader {
///     // Do things with every chunk of 4 bits of data from source_file.txt.
/// }
/// ```
use std::cmp::Eq;
use std::fmt;
use std::fmt::{Display, Debug, Formatter};
use std::fs::File;
// Write import gets a compiler warning. It warns about importing Write is useless but actually
// if I remove Write import I get a compiler error in this module code.
use std::io::{BufReader, Read, Write, Error};
use std::iter::Iterator;
use std::mem::size_of_val;
use std::ops::Add;
// Write import gets a compiler warning. It warns about importing PathBuf is useless but actually
// if I remove PathBuf import I get a compiler error in this module code.
use std::path::PathBuf;

use bitreader::{BitReader, BitReaderError};
use image::open;

use crate::bytetools::u24_to_bytes;


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
impl Add for Remainder {
    type Output = BinaryAccumulationResult;

    fn add(self, rhs: Self) -> Self::Output {
        unimplemented!()
    }
}

impl Display for Remainder {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "(data: {}, length: {})",
               self.data, self.length)
    }
}

/// Type returned after binary accumulate two remainders.
#[derive(PartialEq)]
struct BinaryAccumulationResult {
    complete_byte: Option<u8>,
    remainder: Option<Remainder>,
}

impl Debug for BinaryAccumulationResult  {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let complete_byte = match &self.complete_byte {
            Some(value)=> *value,
            None=> 0,
        };
        let remainder = match &self.remainder {
            Some(remainder)=> (*remainder).clone(),
            None=> Remainder::new(0,0),
        };
        write!(f, "(complete_byte: {}, remainder: {})",
               complete_byte, remainder)
    }
}

/// Wrapper around file contents.
///
/// Once this type is created with its new() method, file is automatically read and its contents
/// is placed at "content" attribute.
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
/// Iterator will try to read self.chunk_size bits at a time. So returned chunk's length attribute
/// is going to be equal to self.chunk_size unless we are really near to the file end. In that
/// last case less than self.chunk_size will be actually read so chunk's length attribute will
/// have the actual number of bits that were actually read.
impl<'a> Iterator for ContentReader<'a> {
    type Item = Chunk;

    fn next(&mut self) -> Option<Self::Item> {
        let chunk = match self.bit_reader.read_u32(self.chunk_size) {
            Ok(bits)=> {
                self.position += 1;
                Chunk::new(bits, self.chunk_size, self.position)
            }
            Err(e)=> {
                if let BitReaderError::NotEnoughData {position: _, length, requested: _} = e {
                    let bits = self.bit_reader.read_u32(length as u8)
                        .expect("Error reading last few bits from file to be hidden.");
                    self.position += 1;
                    Chunk::new(bits, length as u8, self.position)
                } else {
                    panic!("Error reading data to be hidden");
                }
            }
        };
        Some(chunk)
    }
}

/// Wrapper over an open file to write into it chunks extracted from host files.
///
/// Complete bytes are written at once but border bytes need to be rebuild from two different
/// chunks, so we need "pending_byte" to use as a temporal container until it is filled
/// completely and we can write it.
pub struct FileWriter {
    /// Destination file to write chunks into.
    destination: File,
    /// Buffer byte to write into extracted bits until we have a complete byte to write into
    /// destination.
    pending_byte: u8,
    /// How many bits from left we have written so far into pending_byte.
    pending_byte_written_length: u8,
}

impl FileWriter {
    #[must_use]
    pub fn new(destination_file: &str)-> Result<Self, Error> {
        let destination = File::create(destination_file)?;
        Ok(FileWriter{destination, pending_byte: 0, pending_byte_written_length: 0})
    }

    /// Write Chunk into self.destination file.
    ///
    /// Actually only complete bytes will be written into file. Incomplete remainder bytes
    /// will be stored into self.pending_bytes until they fill up.
    pub fn write(&mut self, chunk: Chunk)-> std::io::Result<()>{
        let justified_data = Self::left_justify(chunk.data, chunk.length);
        let complete_bytes = chunk.length / 8;
        for i in 0..complete_bytes {
            self.destination.write(&[justified_data[i as usize]]);
        }
        self.store_remainder(justified_data, chunk.length);
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
    /// let data = 0b_11_u32;
    /// let returned_data = left_justify(data, 2);
    /// assert_eq!(0b_1100_0000_u8, returned_data[0]);
    /// ```
    fn left_justify(data: u32, data_length: u8)-> [u8; 3]{
        let left_shift = 24 - data_length; // Remember 8 leftmost bits are discarded.
        let justified_data = data << left_shift;
        u24_to_bytes(justified_data)
    }

    /// Get bits that do not conform complete bytes.
    ///
    /// # Parameters:
    /// * data: Chunk data already left justified and translated to a 3 bytes long array.
    /// * length: How many bits from left are actual data.
    ///
    /// # Returns:
    /// * A Some(Remainder) if a remainder is available.
    /// * None is returned if there is no remainder available (i.e data conforms an integer
    /// amount of bytes).
    fn get_remainder(data: [u8; 3], length: u8)-> Option<Remainder>{
        let remainder_length = length % 8;
        if remainder_length == 0 {
            None
        } else {
            let complete_bytes = length / 8;
            let remainder_byte = data[complete_bytes as usize];
            let right_shift = 8 - remainder_length;
            let remainder = remainder_byte >> right_shift;
            Some(Remainder::new(remainder, remainder_length))
        }
    }

    /// Keep in self.pending_byte those bits that are not enough to conform a complete byte.
    ///
    /// Bits are accumulated until they fill a byte, then they are written to destination file.
    ///
    /// # Parameters:
    /// * data: Chunk data already left justified and translated to a 3 bytes long array.
    /// * length: How many bits from left are actual data.
    fn store_remainder(&mut self, data: [u8; 3], length: u8){
        if let Some(remainder) = Self::get_remainder(data, length) {
            unimplemented!()
        } else {
            ();
        }
    }
}

impl Drop for FileWriter {
    /// On drop, self.pending_byte content is considered complete and should be stored
    /// into self.destination.
    fn drop(&mut self) {
        unimplemented!()
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use super::super::test_common::TestEnvironment;
    use std::path::Path;
    use std::io::{Cursor, Read};
    use byteorder::{NativeEndian, ReadBytesExt, WriteBytesExt};
    use ring::digest::{Context, Digest, SHA256};

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
    /// * size: Actual sze in bits of data chunk.
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

    /// Hash file content with SHA-256.
    ///
    /// This way we can check to files have same content.
    ///
    /// Original code got from [Rust Cookbok](https://rust-lang-nursery.github.io/rust-cookbook/cryptography/hashing.html)
    fn hash(file_path: &str) -> Result<Digest, Error> {
        let mut reader = BufReader::new(File::open(file_path)?);
        let mut context = Context::new(&SHA256);
        let mut buffer = [0; 1024];

        loop {
            let count = reader.read(&mut buffer)?;
            if count == 0 {
                break;
            }
            context.update(&buffer[..count]);
        }

        Ok(context.finish())
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
            let mut destination_writer = FileWriter::new(destination_file_name_path.as_str())
                .expect("Error happened trying to created FileWriter type.");
            // Transferring chunks.
            for chunk in reader {
                destination_writer.write(chunk);
            }
        }   // Here destination_writer.drop() should have been called so remaining bits should
            // have been written to destination file.
        // Test destination file has same content than source file.
        let source_file_hash = hash(source_path.to_str()
            .expect("Source file name contains odd characters"))
            .expect("Something wrong happened when calculating hash for source file.");
        let destination_file_hash = hash(destination_file_name_path.as_str())
            .expect("Something wrong happened when calculating hash for destination file.");
        assert_eq!(source_file_hash.as_ref(), destination_file_hash.as_ref(),
        "Destination file content is not the same as source file content.");
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
        let remainder_byte = 0b_1011_0000_u8;
        let expected_remainder = 0b_1011_u8;
        let data_1_byte = [remainder_byte, 0, 0];
        let data_1_byte_length = 4;
        let data_2_bytes = [0, remainder_byte, 0];
        let data_2_bytes_length = 12;
        let data_3_bytes = [0, 0, remainder_byte];
        let data_3_bytes_length = 20;
        let remainder1 = FileWriter::get_remainder(data_1_byte, data_1_byte_length)
            .expect("No remainder found");
        assert_eq!((expected_remainder, 4), (remainder1.data, remainder1.length),
                   "We did not get expected remainder when analyzing 1 byte case. Expected {:#?}, but got {:#?}.",
                   (expected_remainder, 4), (remainder1.data, remainder1.length));
        let remainder2 = FileWriter::get_remainder(data_2_bytes, data_2_bytes_length)
            .expect("No remainder found");
        assert_eq!((expected_remainder, 4), (remainder2.data, remainder2.length),
                   "We did not get expected remainder when analyzing 2 byte case. Expected {:#?}, but got {:#?}.",
                   (expected_remainder, 4), (remainder2.data, remainder2.length));
        let remainder3 = FileWriter::get_remainder(data_3_bytes, data_3_bytes_length)
            .expect("No remainder found");
        assert_eq!((expected_remainder, 4), (remainder3.data, remainder3.length),
                   "We did not get expected remainder when analyzing 3 bytes case. Expected {:#?}, but got {:#?}.",
                   (expected_remainder, 4), (remainder3.data, remainder3.length));
    }

    #[test]
    fn test_add_remainder() {
        // Accumulating without overflow.
        let remainder1 = Remainder::new(0b_101_u8, 3);
        let remainder2 = Remainder::new(0b_11_u8, 2);
        let expected_result = BinaryAccumulationResult{
            complete_byte: None,
            remainder: Some(Remainder::new(0b_10111_u8, 5))
        };
        let result = remainder1 + remainder2;
        assert_eq!(expected_result, result,
                   "Accumulation without overflow did not worked as we expected. \
                   We expected a remainder of {:?} but we got {:?}",
                   expected_result, result);
        // Accumulating with overflow.
        let remainder1 = Remainder::new(0b_1010_111_u8, 7);
        let remainder2 = Remainder::new(0b_011_u8, 3);
        let expected_result = BinaryAccumulationResult{
            complete_byte: Some(0b_1010_1110_u8),
            remainder: Some(Remainder::new(0b_11_u8, 2))
        };
        let result = remainder1 + remainder2;
        assert_eq!(expected_result, result,
                   "Accumulation with overflow did not worked as we expected. \
                   We expected a remainder of {:?} but we got {:?}",
                   expected_result, result);
        // Accumulating an exact byte.
        let remainder1 = Remainder::new(0b_1010_111_u8, 7);
        let remainder2 = Remainder::new(0b_0_u8, 1);
        let expected_result = BinaryAccumulationResult{
            complete_byte: Some(0b_1010_1110_u8),
            remainder: None,
        };
        let result = remainder1 + remainder2;
        assert_eq!(expected_result, result,
                   "Accumulation with overflow did not worked as we expected. \
                   We expected a remainder of {:?} but we got {:?}",
                   expected_result, result);
    }
}