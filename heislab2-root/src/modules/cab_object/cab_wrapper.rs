//---------
// Imports
//---------
// public crates
use std::sync::Arc;

// project crates
use crate::modules::system_status::SystemState;
use crate::modules::cab_object::cab::Cab;
use crate::modules::udp_functions::udp_wrapper;
use crate::modules::elevator_object::elevator_init::Elevator;


//-----------
// Functions
//-----------
/// Initializes and returns a elevator cab
pub fn initialize_cab(elev_num_floors: u8, system_state_ref: &Arc<SystemState>, elevator_clone: Elevator) -> std::io::Result<Cab> {
    // create socket addresses
    let inn_addr = udp_wrapper::create_socket_address(3500);
    let out_addr = udp_wrapper::create_socket_address(3600);
    
    // Assign ID matching state.me_id for local IP assignment
    let set_id = system_state_ref.me_id; 
    println!("me id is {}",system_state_ref.me_id);
    
    //Make free cab
    let mut cab = Cab::init(&inn_addr, &out_addr, elev_num_floors, set_id, system_state_ref)?;
    cab.turn_off_lights(elevator_clone);

    Ok(cab)
} 
