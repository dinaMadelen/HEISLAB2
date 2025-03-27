//---------
// Imports
//---------
// public crates
use std::{
    sync::Arc,
    thread::*
};

// project crates
use crate::modules::{
    system_status::SystemState,
    slave_functions::slave::{
        set_new_master, 
        check_master_failure
    },
    udp_functions::udp::UdpHandler
};

pub fn set_master_id(system_state_clone: Arc<SystemState>) -> () {
    let mut known_elevators_locked = system_state_clone.known_elevators.lock().unwrap();
    let mut cab_clone = known_elevators_locked.get_mut(0).unwrap().clone();
    drop(known_elevators_locked);

    set_new_master(&mut cab_clone, &system_state_clone);
}

pub fn print_master_id(system_state_clone: Arc<SystemState>) -> () {
    let master_id_clone = system_state_clone.master_id.lock().unwrap().clone();
    println!("The master is assigned as: {}",master_id_clone);
}

pub fn spawn_master_failure_check_thread(system_state_clone: Arc<SystemState>, udp_handler_clone: Arc<UdpHandler>) -> () {
        spawn(move||{
            check_master_failure(&system_state_clone, &udp_handler_clone);
        });
}