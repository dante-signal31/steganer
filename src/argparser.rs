use argparse::{ArgumentParser, StoreTrue, Store};
use crate::configuration::Configuration;


/// Parse console arguments given when launching steganer.
///
/// Parsed arguments are stored in a Configuration struct that is returned.
pub fn parse_arguments()-> Configuration{
    let mut configuration =  Configuration::new_default();
    {   // If this section is not "scoped" parser keeps mutably borrowed configuration so we
        // wouldn't be able move it out through Ok(configuration).
        let mut parser = ArgumentParser::new();
        parser.set_description("Hide a file inside another... or recovers it.");
        parser.refer(&mut configuration.hidden_file)
            .add_argument("file_hidden", Store, "File to hide or to be extracted.");
        parser.refer(&mut configuration.host_file)
            .add_argument("host_file", Store, "Container file for hidden file.");
        parser.refer(&mut configuration.extract)
            .add_option(&["-x", "--extract"], StoreTrue, "Extract file.");
        parser.parse_args_or_exit();
    }
    configuration
}