use std::thread::*;
use std::time::*;

use crossbeam_channel as cbc;

use driver_rust::elevio;
use driver_rust::elevio::elev as e;

fn main() -> std::io::Result<()> {
    // let elev_num_floors = 4;
    // let elevator = e::Elevator::init("localhost:15657", elev_num_floors)?;
    // println!("Elevator started:\n{:#?}", elevator);

    // let poll_period = Duration::from_millis(25);
    let x = 5;
    let y = 10; 

    Ok(())
}
