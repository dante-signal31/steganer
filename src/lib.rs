use configuration::Configuration;
use std::error::Error;

/// Main function in steganer. It runs its main logic.
pub fn run(config: Configuration)-> Result<(), Box<dyn Error>>{
    if config.extract {
        extract(config.hidden_file, config.host_file)
    } else {
        hide(config.hidden_file, config.host_file)
    }
}

/// Create a configuration struct.
///
/// This function is only useful for integration tests in order to create configurations to test
/// run function.
pub fn create_configuration(hidden_file: String, host_file: String, extract: bool)-> Configuration {
    Configuration::new(hidden_file, host_file, extract)
}

fn extract(hidden_file: String, host_file: String)-> Result<(), Box<dyn Error>>{
    Ok(())
}

fn hide(hidden_file: String, host_file: String)-> Result<(), Box<dyn Error>>{
    Ok(())
}