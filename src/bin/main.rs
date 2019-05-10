extern crate steganer;

use steganer::argparser::parse_arguments;
use steganer::_run;


fn main() {
    let config = parse_arguments();
    println!("Parsed data: {:?}", config);
    _run(&config);
}

