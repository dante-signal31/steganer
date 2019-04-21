extern crate steganer;

use steganer::argparser::parse_arguments;
use steganer::run;


fn main() {
    let config = parse_arguments();
    println!("Parsed data: {:?}", config);
    run(&config);
}

