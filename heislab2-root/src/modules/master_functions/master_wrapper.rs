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

pub fn set_master_id(system_state_clone: Arc<SystemState>) -> Result<()> {
    let mut known_elevators_locked = system_state_clone.known_elevators.lock()?;
    let mut cab_clone = known_elevators_locked.get_mut(0).unwrap().clone();
    drop(known_elevators_locked);

    set_new_master(&mut cab_clone, &system_state_clone);

    Ok(())
}
