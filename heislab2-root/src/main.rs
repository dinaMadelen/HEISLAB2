#![allow(warnings)]

pub mod udp;
pub mod elevator;
pub mod system_init;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Hello, world!");
    
    Ok(())
}
