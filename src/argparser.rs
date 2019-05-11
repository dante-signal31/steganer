use clap::{Arg, App};
use crate::configuration::Configuration;

fn get_version()-> String {
    format!("{}.{}.{}{}",
            env!("CARGO_PKG_VERSION_MAJOR"),
            env!("CARGO_PKG_VERSION_MINOR"),
            env!("CARGO_PKG_VERSION_PATCH"),
            option_env!("CARGO_PKG_VERSION_PRE").unwrap_or(""))
}

/// Parse console arguments given when launching steganer.
///
/// Parsed arguments are stored in a Configuration struct that is returned.
pub fn parse_arguments()-> Configuration{
    let mut configuration =  Configuration::new_default();
    let matches = App::new("steganer")
        .version(get_version().as_str())
        .author("Dante Signal31 <dante.signal31@gmail.com>")
        .about("Hide a file inside another... or recovers it.")
        .arg(Arg::with_name("file_hidden")
            .help("File to hide or to be extracted.")
            .required(true)
            .value_name("FILE_HIDDEN")
            .index(1)
            .takes_value(true))
        .arg(Arg::with_name("host_file")
            .help("Container file for hidden file.")
            .required(true)
            .value_name("HOST_FILE")
            .index(2)
            .takes_value(true))
        .arg(Arg::with_name("extraction_mode")
            .help("Extracts hidden file (steganer defaults to hide file)")
            .short("x")
            .long("extract"))
        .get_matches();
    configuration.hidden_file = String::from(matches.value_of("file_hidden").unwrap());
    configuration.host_file = String::from(matches.value_of("host_file").unwrap());
    configuration.extract =if matches.is_present("extraction_mode") {true} else {false};
    configuration
}