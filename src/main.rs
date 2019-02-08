// To use ? operator with Options, as this feature is still in nightly channel.
#![feature(try_trait)]

mod argparser;
mod configuration;

use crate::argparser::parse_arguments;

fn main() {
    let config = parse_arguments().expect("Error parsing arguments");
    println!("Parsed data: {:?}", config);
}

