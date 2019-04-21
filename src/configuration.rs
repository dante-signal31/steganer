/// Configuration to make run an steganer execution properly.
#[derive(Debug)]
pub struct Configuration {
    /// If *self.extract* is *true* then *self.hidden_file* gives the name of the file to create to
    /// put extracted hidden data into. Conversely, if *self.extract" is *false* then
    /// *self.hidden_file* points to the file whose content must be hidden.
    pub hidden_file: String,
    /// Name of file where data must be hidden or recovered from depending of whereas *self.extract*
    /// is *true* or *false*.
    pub host_file: String,
    /// Set if this operation is going to hide data or extract it.
    pub extract: bool,
}

impl Configuration{
    /// Create an empty Configuration struct.
    ///
    /// String attributes of this struct will br initialized to an empty string. Extract to false.
    /// To initialize attributtes set them directly after creation.
    pub fn new_default() -> Self {
        Configuration{ hidden_file: "".to_owned(), host_file: "".to_owned(), extract: false}
    }

    /// Create a Configuration struct with given attributes.
    #[must_use]
    pub fn new(hidden_file: &str, host_file: &str, extract: bool)-> Self {
        Configuration{hidden_file: hidden_file.to_owned(), host_file: host_file.to_owned(), extract}
    }
}