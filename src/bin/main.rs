extern crate steganer;

use steganer::*;
use steganer::argparser::parse_arguments;
use steganer::_run;


fn main() {
    let config = parse_arguments();
    if let Err(ref errors) = _run(&config) {
        eprintln!("Error found. Execution aborted.");
        eprintln!("Error details: ");
        errors.iter()
            .enumerate()
            .for_each(|(index, error)| eprintln!("\t {} --> {}", index, error));
        if let Some(backtrace) = errors.backtrace(){
            eprintln!("{:?}", backtrace);
        }
        std::process::exit(1);
    } else {
        std::process::exit(0)
    }
}

