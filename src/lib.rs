pub mod argparser;
mod bytetools;
mod configuration;
mod fileio;
mod stegimage;

use std::fs::metadata;
use std::ops::Add;

use error_chain::{error_chain, bail};
use pyo3::prelude::*;
use pyo3::{wrap_pyfunction, PyErr, exceptions};

use crate::configuration::Configuration;
use crate::fileio::{FileContent, ContentReader, FileWriter};
use crate::stegimage::ContainerImage;

// This will create the Error, ErrorKind, ResultExt, and Result types.
error_chain!{}

/// Main function in steganer. It runs its main logic.
///
/// If you're using steganer as a library then this function is not useful for you.
pub fn _run(config: &Configuration) -> Result<()> {
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
pub fn _create_configuration(hidden_file: &str, host_file: &str, extract: bool) -> Configuration {
    Configuration::new(hidden_file, host_file, extract)
}

/// Extract a file hidden into an image using steganography techniques.
///
/// # Parameters:
/// * hidden_file: Absolute path to file to hide.
/// * host_file: Absolute path to image file that is going to contain hidden file.
pub fn extract_from_image(hidden_file: &str, host_file: &str)-> Result<()> {
    let mut host_image = ContainerImage::new(host_file)?;
    host_image.setup_hidden_data_extraction();
    let mut extracted_file = FileWriter::new(hidden_file)
        .chain_err(||"Error creating destination file to store extracted data")?;
    for chunk in host_image {
        extracted_file.write(&chunk)?;
    }
    Ok(())
}

/// Exported version of extract_from_image() for python module.
///
/// # Parameters:
/// * hidden_file: Absolute path to file to hide.
/// * host_file: Absolute path to image file that is going to contain hidden file.
#[pyfunction]
fn unhide_from_image(hidden_file: &str, host_file: &str)-> PyResult<()> {
    match extract_from_image(hidden_file, host_file) {
        Ok(())=> Ok(()),
        Err(ref errors)=> {
            let mut message = String::new();
            for (index, error) in errors.iter().enumerate() {
                message = message.add(format!("\t {} --> {}", index, error).as_str());
            }
            Err(PyErr::new::<exceptions::IOError, _>(message))
        },
    }
}

/// Hide a file into into an image using steganography techniques.
///
/// # Parameters:
/// * file_to_hide: Absolute path to hidden file.
/// * host_file: Absolute path to image file that contains hidden file.
pub fn hide_into_image(file_to_hide: &str, host_file: &str)-> Result<()> {
    let file_to_hide_content = FileContent::new(file_to_hide)
        .chain_err(||"Error creating file to hide content handle.")?;
    let file_to_hide_size = metadata(file_to_hide)
        .chain_err(||"Error accessing file to hide metadata.")?
        .len();
    if file_to_hide_size > std::u32::MAX as u64 {
        bail!("File to hide is too big. Maximum size is {}", std::u32::MAX);
    } else {
        let mut host_image = ContainerImage::new(host_file)?;
        let chunk_size = host_image.setup_hiding(file_to_hide_size as u32);
        let file_to_hide_reader = ContentReader::new(&file_to_hide_content, chunk_size);
        for chunk in file_to_hide_reader {
            host_image.hide_data(&chunk);
        }
    }
    Ok(())
}

/// Exported version of hide_into_image() for python module.
///
/// # Parameters:
/// * file_to_hide: Absolute path to hidden file.
/// * host_file: Absolute path to image file that contains hidden file.
#[pyfunction]
fn hide_inside_image(file_to_hide: &str, host_file: &str)-> PyResult<()> {
    match hide_into_image(file_to_hide, host_file) {
        Ok(())=> Ok(()),
        Err(ref errors)=> {
            let mut message = String::new();
            for (index, error) in errors.iter().enumerate() {
                message = message.add(format!("\t {} --> {}", index, error).as_str());
            }
            Err(PyErr::new::<exceptions::IOError, _>(message))
        },
    }
}

/// Export to create a steganer python module.
#[pymodule]
fn steganer(_py: Python, m: &PyModule)-> PyResult<()>{
    m.add_wrapped(wrap_pyfunction!(unhide_from_image))?;
    m.add_wrapped(wrap_pyfunction!(hide_inside_image))?;
    Ok(())
}