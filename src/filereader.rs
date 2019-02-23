//use std::fs::File;
//use std::io::{BufReader, Write, Error};
//use std::iter::{Iterator, Enumerate};
//use std::path::PathBuf;
//use std::io::BufReader;
use std::io::Cursor;
use byteorder::{NativeEndian, ReadBytesExt, WriteBytesExt};

struct chunk {
    data_bytes: [u8; 3],
    data_int: u32,
    order: u64,
    odd: bool,
}

impl chunk {
    #[must_use]
    pub fn new(bytes: Vec<u8>, size: u8, order: u64)-> Self {
        let odd = if order%2 == 0 {false} else {true};
        let (data_bytes, data_int) = normalize_data(bytes, size, odd);
        chunk{data_bytes: [data_bytes[0], data_bytes[1], data_bytes[2]], data_int, order, odd}
    }


}

fn normalize_data(bytes: &Vec<u8>, size: u8, odd: bool)-> ([u8; 4], u32){
    let mut rdr = Cursor::new(bytes);
    let mut int_value = rdr.read_u32::<NativeEndian>().expect("Error converting read bytes to u32");
    if odd {
        let shift = 32 - size;
        int_value = int_value >> shift;
    }
    let mut bytes_value: Vec<u8> = vec![];

    bytes_value.write_u32::<NativeEndian>(int_value).expect("Error encoding normalized bytes to u32");
    let bytes_array = [bytes_value[0], bytes_value[1], bytes_value[2], bytes_value[3]];
    (bytes_array, int_value)
}

//struct FileReader {
//    source: File,
//    reader: BufReader,
//    chunk_size: u8,
//    chunk_complete_bytes: u8,
//    chunk_remainder_size: u8,
//    post_remainder: u8,
//    even_chunk: bool, // Even chunks have their remainder in the last byte.
//    position: u64
//}

//impl FileReader {
//    #[must_use]
//    pub fn new(source_file: &str, chunk_size: u8)-> Result<Self, Error> {
//        Ok(FileReader {
//            source: File::open(source_file)?,
//            reader: BufReader.new(source),
//            chunk_size,
//            chunk_complete_bytes: chunk_size / 8,
//            chunk_remainder_size: chunk_size % 8,
//            post_remainder: 0,
//            even_chunk: true,
//            position: 0,
//        })
//    }
//}
//
//impl Iterator for Filereader {
//    type Item = Vec<u8>;
//
//    fn next(&mut self) -> Option<Self::Item> {
//        let mut buffer_read = match chunk_remainder {
//            0=> [0u8; self.chunk_complete_bytes],
//            _=> [0u8; self.chunk_complete_bytes+1],
//        };
//        self.reader.read_exact(&mut buffer_read);
//        let mut chunk = buffer_read.to_vec();
//        if self.even_chunk {
//            let mut last_position = chunk.last_mut()
//                .expect("Could not access to last position of read chunk.");
//            self.post_remainder =
//
//        }
//    }
//
//    fn enumerate(self) -> Enumerate<Self> where Self: Sized {
//        unimplemented!()
//    }
//}


#[cfg(test)]
mod tests {
    use super::*;
    use super::super::test_common::TestEnvironment;
    use std::path::Path;

    const MESSAGE: String = "Lorem ipsum dolor sit amet, consectetur adipiscing elit, \
    sed eiusmod tempor incidunt ut labore et dolore magna aliqua.".to_owned();
    const SOURCE_FILE: String = "source.txt".to_owned();

    fn populate_test_file(test_env: &TestEnvironment) -> PathBuf {
        let source_path = Path::new(test_env.path()).join(SOURCE_FILE);
        let mut source_file = File::create(source_path)
            .expect("Could not create test source file");
        source_file.write_all(&MESSAGE.into_bytes())
            .expect("Error populating test source file.");
        source_path
    }

    #[test]
    // Test iteration with chunks smaller than 8 bits.
    fn test_iterator_next_under_8() {
        let test_env = TestEnvironment::new();
        let source_path = populate_test_file(&test_env);
        let reader = FileReader::new(source_path.to_str()
                                         .expect("Source file name contains odd characters."),
                                     4)
            .expect("There was a problem reading source file.");
        let mut chunk = reader.next(); // Upper half of "L".
        let mut expected_chunk = "L".to_owned().into_bytes();
        expected_chunk[0] = expected_chunk[0] & 0xF0;
        assert_eq!(expected_chunck, chunk);
        reader.next(); // Lower half of "L".
        reader.next(); // Upper half of "o".
        chunk = reader.next(); // Lower half of "o".
        expected_chunk = "o".to_owned().into_bytes();
        expected_chunk[0] = expected_chunk[0] & 0x0F;
        assert_eq!(expected_chunk, chunk);
    }

    #[test]
    // Test iteration with chunks bigger than 8 bits.
    fn test_iterator_next_over_8() {
        let test_env = TestEnvironment::new();
        let source_path = populate_test_file(&test_env);
        let reader = FileReader::new(source_path.to_str()
            .expect("Source file name contains odd characters."),
                                     12)
            .expect("There was a problem reading source file.");
        let mut chunk = reader.next(); // "L" and upper half of "o".
        let mut expected_chunk = "Lo".to_owned().into_bytes();
        expected_chunk[1] = expected_chunk[1] & 0xF0;
        assert_eq!(expected_chunk, chunk);
        reader.next(); // Lower half of "o" and "r".
        reader_next(); // "e" and upper half of "m".
        chunk = reader_next(); // Lower half "m" and " "
        expected_chunk = "m ".to_owned().into_bytes();
        expected_chunk[0] = expected_chunk[0] & 0x0F;
        assert_eq!(expected_chunk, chunk);
    }

    #[test]
    fn test_normalize_data_smaller_than_8() {
        // Odd case.
        let mut raw_data = [0b11110000u8, 0b00000000u8, 0b00000000u8, 0b00000000u8];
        let (normalized_bytes, data_int) = normalize_data(&raw_data.to_vec(), 4, true);
        assert_eq!([0b00000000u8, 0b00000000u8, 0b00000000u8, 0b00001111u8], normalized_bytes);
        assert_eq!(0b00001111u32, data_int);
        // Even case.
        raw_data = [0b00001111u8, 0b00000000u8, 0b00000000u8, 0b00000000u8];
        let (normalized_bytes, data_int) = normalize_data(&raw_data.to_vec(), 4, false);
        assert_eq!([0b00000000u8, 0b00000000u8, 0b00000000u8, 0b00001111u8], normalized_bytes);
        assert_eq!(0b00001111u32, data_int);
    }

    #[test]
    fn test_normalize_data_bigger_than_8() {
        // Odd case.
        let mut raw_data = [0b11111111u8, 0b11110000u8, 0b00000000u8, 0b00000000u8];
        let (normalized_bytes, data_int) = normalize_data(&raw_data.to_vec(), 12, true);
        assert_eq!([0b00000000u8, 0b00000000u8, 0b00001111u8, 0b11111111u8], normalized_bytes);
        assert_eq!(0b00001111_11111111u32, data_int);
        // Even case.
        raw_data = [0b00001111u8, 0b11111111u8, 0b00000000u8, 0b00000000u8];
        let (normalized_bytes, data_int) = normalize_data(&raw_data.to_vec(), 12, false);
        assert_eq!([0b00000000u8, 0b00000000u8, 0b00001111u8, 0b11111111u8], normalized_bytes);
        assert_eq!(0b00001111_11111111u32, data_int);
    }

}