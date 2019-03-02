use std::fs::File;
use std::io::{BufReader, Read, Write, Error};
use std::iter::Iterator;
use bitreader::BitReader;
use std::path::PathBuf;
//use std::io::BufReader;


pub struct Chunk {
    pub data: u32,
    pub order: u64,
}

impl Chunk {
    #[must_use]
    pub fn new(data: u32, order: u64)-> Self {
        Chunk {data, order}
    }
}

struct FileContent {
    source: File,
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

struct ContentReader<'a> {
    bit_reader: BitReader<'a>,
    chunk_size: u8,
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
    fn normalize(data: u32, size: u8)-> u32{
        let shift = 32 - size;
        data >> shift
    }

    #[test]
    // Test iteration with chunks smaller than 8 bits.
    fn test_iterator_next_under_8() {
        let test_env = TestEnvironment::new();
        let source_path = populate_test_file(&test_env);
        let file_content = FileContent::new(source_path.to_str()
            .expect("Source file name contains odd characters."))
            .expect("Error getting file contents");
        let mut reader = ContentReader::new(&file_content, 4)
            .expect("There was a problem reading source file.");
        let mut chunk: Chunk = reader.next()
            .expect("Error reading chunk"); // Upper half of "L".
        let mut expected_chunk = "L".to_owned().as_bytes()[0] as u32;
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
        let test_env = TestEnvironment::new();
        let source_path = populate_test_file(&test_env);
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
        // expected_chunk = 0b0110_0000_0100_1100
        expected_chunk = expected_chunk & 0xF0FF;
        let mut wtr: Vec<u8> = Vec::new();
        // wtr = [0100_1100, 0110_0000, 0, 0]
        wtr.write_u32::<NativeEndian>(expected_chunk)
            .expect("Error writing chunk bigger than 8.");
        let mut expected_int = normalize(bytes_to_u32(wtr), 12);
        // expected_int = 0b0100_1100_0110_0000_0000_0000_0000_0000
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
        // expected_chunk = 0b0010_0000_0000_1101
        expected_chunk = expected_chunk & 0xFF0F;
        wtr = Vec::new();
        // wtr = [0000_1101, 0010_0000, 0, 0]
        wtr.write_u32::<NativeEndian>(expected_chunk)
            .expect("Error writing chunk bigger than 8.");
        // expected_int = 0b0000_1101_0010
        expected_int = normalize(bytes_to_u32(wtr), 12);
        // chunk_data = 0b1101_0010_0000
        assert_eq!(expected_int, chunk.data);
    }
}