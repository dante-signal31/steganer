// To use ? operator with Options, as this feature is still in nightly channel.
//#![feature(try_trait)]
use crate::lib::argparser::parse_arguments;
use crate::lib::run;

fn main() {
    let config = parse_arguments();
    println!("Parsed data: {:?}", config);
    run(config);
}

