use crossbeam_channel as cbc;
use std::{
    thread::*,
    time::*,
    sync::Arc,

};

use crate::modules::{
    cab_object::elevator_status_functions::Status,
    slave_functions::slave::*,
    master_functions::master::*,
    udp_functions::udp_handler_init::*,
    udp_functions::udp::*,
    system_status::*,
};

pub fn spawn_master_monitor(system_state_clone: Arc<SystemState>, udp_handler_clone: Arc<UdpHandler>){
    spawn(move|| {
        loop{
            fix_master_issues(&system_state_clone, &udp_handler_clone);

            sleep(Duration::from_secs(1));
            let now = SystemTime::now();
            
            // Iterate in reverse order so that removing elements doesn't affect things
            {
                let  known_elevators_locked = system_state_clone.known_elevators.lock().unwrap();
                for i in (0..known_elevators_locked.len()).rev() {
                    let elevator = &known_elevators_locked[i];
                    // Only check elevators that are Moving or DoorOpen.
                    if elevator.status == Status::Moving || elevator.status == Status::DoorOpen {
                        if let Ok(elapsed) = now.duration_since(elevator.last_lifesign) {
                            if elapsed >= Duration::from_secs(10) {
                                let dead_elevator = known_elevators_locked.get(i).unwrap();
                                println!("Elevator {} is dead (elapsed: {:?})", dead_elevator.id, elapsed);
                                let error_offline_msg = make_udp_msg(system_state_clone.me_id, MessageType::ErrorOffline, UdpData::Cab(dead_elevator.clone()));
                                for elevator in known_elevators_locked.iter(){
                                    udp_handler_clone.send(&elevator.inn_address, &error_offline_msg);
                                }
                            }
                        }
                    }
                }
            }
            

            {   
                let locked_master_id = system_state_clone.master_id.lock().unwrap();
                let worldview_system_state=Arc::clone(&system_state_clone);
                if system_state_clone.me_id == *locked_master_id{
                    drop(locked_master_id);
                    print!("BROADCASTING WORLDVIEW _____________________");
                    //MASTER WORLDVIEW BROADCAST
                    master_worldview(&worldview_system_state, &udp_handler_clone.clone());
                }
            }
            sleep(Duration::from_secs(1));
            check_master_failure(&system_state_clone, &udp_handler_clone);
        }
});

}

pub fn spawn_queue_finisher(elevator_clone: Elevator,system_state_clone: Arc<SystemState>,  door_tx_clone: cbc::Sender<bool>,obstruction_rx_clone: cbc::Receiver<bool>){
    spawn(move|| {
        loop{
            sleep(Duration::from_millis(300));
            
            let mut known_elevators_locked = system_state_clone.known_elevators.lock().unwrap();
            if !known_elevators_locked.get_mut(0).unwrap().queue.is_empty(){
                known_elevators_locked.get_mut(0).unwrap().go_next_floor(door_tx_clone.clone(), obstruction_rx_clone.clone() ,elevator_clone.clone());
                
            }
            drop(known_elevators_locked);

            let mut known_elevators_locked = system_state_clone.known_elevators.lock().unwrap().clone();
            known_elevators_locked.get_mut(0).unwrap().lights(&system_state_clone.clone(), elevator_clone.clone());
            known_elevators_locked.get_mut(0).unwrap().print_status();
            elevator_clone.floor_indicator(known_elevators_locked.get_mut(0).unwrap().current_floor);
            

        }
    });
}