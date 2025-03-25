//---------
// Imports
//---------

use crate::modules::system_status::SystemState;
use std::sync::Arc;
use crate::modules::cab_object::cab::Cab;
use crate::modules::udp_functions::udp_wrapper;

pub fn create_cab(elev_num_floors_ref: &u8, system_state_ref: &Arc<SystemState>) -> Cab {
    // create socket addresses
    let inn_addr = udp_wrapper::create_socket_address(3500);
    let out_addr = udp_wrapper::create_socket_address(3600);
    
    // Assign ID matching state.me_id for local IP assignment
    let set_id = system_state.me_id; 
    println!("me id is {}",system_state.me_id);
    
    //Make free cab
    let mut cab = Cab::init(&inn_addr, &out_addr, elev_num_floors, set_id, &system_state)?;
    cab.turn_off_lights(elevator.clone());
} 
