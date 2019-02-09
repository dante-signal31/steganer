use configuration::Configuration;
use std::error::Error;

/// Main function in steganer. It runs its main logic.
pub fn run(config: Configuration)-> Result<(), Box<dyn Error>>{
    if config.extract {
        extract(config)
    } else {
        hide(config)
    }
}

/// Create a configuration struct.
///
/// This function is only useful for integration tests in order to create configurations to test
/// run function.
pub fn create_configuration(file_hidden: String, host_file: String, extract: bool)-> Configuration {
    Configuration::new(file_hidden, host_file, extract)
}

fn extract(config: Configuration)-> Result<(), Box<dyn Error>>{
    Ok(())
}

fn hide(config: Configuration)-> Result<(), Box<dyn Error>>{
    Ok(())
}