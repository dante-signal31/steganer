pub mod argparser;
pub mod bytetools;
pub mod configuration;
pub mod fileio;
pub mod stegimage;

use std::error::Error;
use std::fs::{File, metadata};
use std::io::Result;

use crate::configuration::Configuration;
use crate::fileio::{FileContent, ContentReader, FileWriter};
use crate::stegimage::ContainerImage;

/// Main function in steganer. It runs its main logic.
pub fn run(config: &Configuration)-> Result<()> {
    if config.extract {
        extract_from_image(&config.hidden_file, &config.host_file)
    } else {
        hide_into_image(&config.hidden_file, &config.host_file)
    }
}

/// Create a configuration struct.
///
/// This function is only useful for integration tests in order to create configurations to test
/// run function.
pub fn create_configuration(hidden_file: &str, host_file: &str, extract: bool)-> Configuration {
    Configuration::new(hidden_file, host_file, extract)
}

/// Extract a file hidden into an image using steganography techniques.
///
/// # Parameters:
/// * hidden_file: Absolute path to file to hide.
/// * host_file: Absolute path to image file that is going to contain hidden file.
pub fn extract_from_image(hidden_file: &str, host_file: &str)-> Result<()> {
    let mut host_image = ContainerImage::new(host_file);
    host_image.setup_hidden_data_extraction();
    let mut extracted_file = FileWriter::new(hidden_file)
        .expect("Error creating destination file for extracted data");
    for chunk in host_image {
        extracted_file.write(&chunk)
            .expect("Error writing extracted data to destination file");
    }
    Ok(())
}

/// Hide a file into into an image using steganography techniques.
///
/// # Parameters:
/// * file_to_hide: Absolute path to hidden file.
/// * host_file: Absolute path to image file that contains hidden file.
pub fn hide_into_image(file_to_hide: &str, host_file: &str)-> Result<()> {
    let file_to_hide_content = FileContent::new(file_to_hide)
        .expect("Error reading file to hide content.");
    let file_to_hide_size = metadata(file_to_hide)
        .expect("Error accessing file to hide metadata.")
        .len();
    if file_to_hide_size > std::u32::MAX as u64 {
        panic!("File to hide is too big. Maximum size is {}", std::u32::MAX);
    } else {
        let mut host_image = ContainerImage::new(host_file);
        let chunk_size = host_image.setup_hiding(file_to_hide_size as u32);
        let file_to_hide_reader = ContentReader::new(&file_to_hide_content, chunk_size)
            .expect("Error getting an iterator to file to hide content.");
        for chunk in file_to_hide_reader {
            host_image.hide_data(&chunk);
        }
    }
    Ok(())
}