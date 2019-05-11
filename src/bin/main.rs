extern crate steganer;

use steganer::argparser::parse_arguments;
use steganer::_run;


fn main() {
    let config = parse_arguments();
    println!("Parsed data: {:?}", config);
    match _run(&config) {
        Ok(())=> std::process::exit(0),
        _ => std::process::exit(1),
    };
}

