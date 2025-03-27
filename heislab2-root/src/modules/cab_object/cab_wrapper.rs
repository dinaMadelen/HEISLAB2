//---------
// Imports
//---------
// public crates
use std::sync::Arc;
use std::os::unix::net::SocketAddr;

// project crates
use crate::modules::system_status::SystemState;
use crate::modules::cab_object::cab::Cab;
use crate::modules::udp_functions::udp_wrapper;
use crate::modules::elevator_object::elevator_init::Elevator;


//-----------
// Functions
//-----------
/// Initializes and returns a elevator cab
pub fn initialize_cab(
    elev_num_floors: u8, 
    system_state_ref: &Arc<SystemState>, 
    elevator_clone: Elevator, 
    in_addr: SocketAddr, 
    out_addr:SocketAddr
) -> std::io::Result<Cab> {
    // Assign ID matching state.me_id for local IP assignment
    let set_id = system_state_ref.me_id; 
    println!("me id is {}",system_state_ref.me_id);
    
    //Make free cab
    let mut cab = Cab::init(&in_addr, &out_addr, elev_num_floors, set_id, system_state_ref)?;
    cab.turn_off_lights(elevator_clone);

    Ok(cab)
} 

/// Pushes a newly created cab to system state
pub fn add_cab_to_sys_state(sys_state_clone: Arc<SystemState>, cab: Cab) -> Result<()> {
    let mut known_elevators_locked = system_state_clone.known_elevators.lock()?;
    known_elevators_locked.push(cab);
    drop(known_elevators_locked);

    println!("Cab initialized:\n{:#?}", elevator);
    Ok(())
}
