pub mod argparser;
pub mod configuration;
pub mod filereader;
pub mod test_common;

use std::error::Error;
use std::fs::File;

use crate::configuration::Configuration;


/// Main function in steganer. It runs its main logic.
pub fn run(config: &Configuration)-> Result<(), Box<dyn Error>>{
    if config.extract {
        extract(&config.hidden_file, &config.host_file)
    } else {
        hide(&config.hidden_file, &config.host_file)
    }
}

/// Create a configuration struct.
///
/// This function is only useful for integration tests in order to create configurations to test
/// run function.
pub fn create_configuration(hidden_file: String, host_file: String, extract: bool, chunk_size: u8)-> Configuration {
    Configuration::new(hidden_file, host_file, extract, chunk_size)
}

/// Encode a file into another using steganography techniques.
///
/// Returns a boxed Error if something bad happens.
///
/// # Parameters:
/// * hidden_file: Absolute path to file to hide.
/// * host_file: Absolute path to file that is going to contain hidden file.
pub fn extract(hidden_file: &String, host_file: &String)-> Result<(), Box<dyn Error>>{
    let file_to_hide = File::open(hidden_file)
        .expect("Could not open hidden file.");
    Ok(())
}

/// Decode a file hidden into another using steganography techniques.
///
/// Returns a boxed Error if something bad happens.
///
/// # Parameters:
/// * hidden_file: Absolute path to hidden file.
/// * host_file: Absolute path to file that contains hidden file.
pub fn hide(hidden_file: &String, host_file: &String)-> Result<(), Box<dyn Error>>{
    Ok(())
}