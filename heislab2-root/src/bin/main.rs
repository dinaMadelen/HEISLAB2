#![allow(warnings)]

use heislab2_root::udp;
use heislab2_root::elevator;
use heislab2_root::system_init;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Hello, world! Signed {}", file!());
    Ok(())
}
