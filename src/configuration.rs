/// Configuration to make run right an steganer execution.
#[derive(Debug)]
pub struct Configuration {
    pub file_hidden: String,
    pub host_file: String,
    pub extract: bool,
}

impl Configuration{
    /// Create an empty Configuration struct.
    ///
    /// String attributes of this struct will br initialized to an empty string. Extract to false.
    /// To initialize attributtes set them directly after creation.
    pub fn new_default() -> Self {
        Configuration{file_hidden: "".to_owned(), host_file: "".to_owned(), extract: false}
    }

    /// Create a Configuration struct with given attributes.
    pub fn new(file_hidden: String, host_file: String, extract: bool)-> Self {
        Configuration{file_hidden, host_file, extract}
    }
}