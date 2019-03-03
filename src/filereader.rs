/// Module to read file to hide contents.
///
/// Thanks to ContentReader type you can get an iterator to read a file to hide a get its bits
/// in predefined bunches. Every bunch of bits are returned inside a Chunk type.
///
/// # Usage example:
/// ```
/// use steganer::filereader::{FileContent, ContentReader};
///
/// let file_content = FileContent::new("source_file.txt");
/// let reader = ContentReader::new(&file_content, 4);
/// for chunk in reader {
///     // Do things with every chunk of 4 bits of data from source_file.txt.
/// }
/// ```
use std::fs::File;
use std::io::{BufReader, Read, Write, Error};
use std::iter::Iterator;
use bitreader::BitReader;
use std::path::PathBuf;

/// Bits read from files to be hidden are stored at Chunks.
pub struct Chunk {
    /// Every Chunk stores a maximum of 32 read bits at this property, those bits are
    /// at natural order (Big Endian) and justified to right.
    pub data: u32,
    /// An index about relative position of this chunk at file to be hidden.
    pub order: u64,
}

impl Chunk {
    #[must_use]
    pub fn new(data: u32, order: u64)-> Self {
        Chunk {data, order}
    }
}

/// Wrapper around file contents.
///
/// Once this type is created with its new() method, file is automatically read and its contents
/// is placed at "content" attribute.
struct FileContent {
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
        buf_reader.read_to_end(&mut content);
        Ok(FileContent {
            source,
            content,
        })
    }
}

/// ContentReader gives you an iterator to read a FileContent data.
///
/// Iterator returns a Chunk Type with bit read in every read iteration.
struct ContentReader<'a> {
    /// BitReader type to read bits alone.
    bit_reader: BitReader<'a>,
    /// Amount of bits to get in each iterator round.
    chunk_size: u8,
    /// Index about how many read rounds we've done using iterator.
    position: u64,
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

impl<'a> Iterator for ContentReader<'a> {
    type Item = Chunk;

    fn next(&mut self) -> Option<Self::Item> {
        let data = self.bit_reader.read_u32(self.chunk_size)
            .expect("Error reading data");
        self.position += 1;
        let chunk = Chunk::new(data, self.position);
        Some(chunk)
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use super::super::test_common::TestEnvironment;
    use std::path::Path;
    use std::io::Cursor;
    use byteorder::{NativeEndian, ReadBytesExt, WriteBytesExt};
    const MESSAGE: &str = "Lorem ipsum dolor sit amet, consectetur adipiscing elit, \
    sed eiusmod tempor incidunt ut labore et dolore magna aliqua.";
    const SOURCE_FILE: &str = "source.txt";

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
}