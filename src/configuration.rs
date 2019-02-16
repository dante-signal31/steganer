/// Configuration to make run right an steganer execution.
#[derive(Debug)]
pub struct Configuration {
    pub hidden_file: String,
    pub host_file: String,
    pub extract: bool,
    pub chunk_size: u8,
}

impl Configuration{
    /// Create an empty Configuration struct.
    ///
    /// String attributes of this struct will br initialized to an empty string. Extract to false.
    /// To initialize attributtes set them directly after creation.
    pub fn new_default() -> Self {
        Configuration{ hidden_file: "".to_owned(), host_file: "".to_owned(), extract: false, chunk_size: 1}
    }

    /// Create a Configuration struct with given attributes.
    pub fn new(hidden_file: String, host_file: String, extract: bool, chunk_size: u8)-> Self {
        Configuration{ hidden_file, host_file, extract, chunk_size}
    }
}