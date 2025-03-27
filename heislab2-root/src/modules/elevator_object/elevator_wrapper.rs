use std::time::Duration;

use crate::modules::{elevator_object::elevator_init::Elevator, io::io_init::IoChannels};
use super::elevator_init::SystemState;

/// Goes down until a floor is found
pub fn go_down_until_floor_found(elevator: &mut Elevator, dirn: u8) -> () {
    if elevator.floor_sensor().is_none() {
        elevator.motor_direction(dirn);
    }
}

pub fn spawn_elevator_monitor_thread(system_state_clone: Arc<SystemState>, udp_handler_clone: Arc<UdpHandler>) -> () {
    // threshold for when elevator is considered dead
    const DEAD_ELEV_THRESHOLD:Duration = Duration::from_secs(10);
        spawn(move||{
                loop{
                    fix_multiple_masters_lowest_id_is_master(&system_state_clone);
                    sleep(Duration::from_secs(3));
                    let now = SystemTime::now();
                    
                    // get known elevators
                    let  known_elevators_locked = system_state_clone.known_elevators.lock().unwrap();
                    // reverse the list
                    let reverse_known_elevators_locked = (0..known_elevators_locked.len()).rev();

                    // iterate over the reversed list
                    for elev_index in reverse_known_elevators_locked {
                        // get elevator
                        let elevator = &known_elevators_locked[elev_index];

                        // skip iteration if elevator isn't moving or the door is open
                        if !(elevator.status == Status::Moving || elevator.status == Status::DoorOpen) {continue;}
                        
                        // only executes if duration since last lifesign is fetched
                        if let Ok(elapsed) = now.duration_since(elevator.last_lifesign) {
                            // if time elapsed is below threshold, skip current iteration
                            if (elapsed < DEAD_ELEV_THRESHOLD) {continue;} 
                            
                            // get dead elevator
                            let dead_elevator = known_elevators_locked.get(elev_index).unwrap();
                            println!("Elevator {} is dead (elapsed: {:?})", dead_elevator.id, elapsed);

                            // transmit dead elevator over udp
                            let msg = make_udp_msg(system_state_clone.me_id, MessageType::ErrorOffline, UdpData::Cab(dead_elevator.clone()));
                            for elevator in known_elevators_locked.iter(){
                                udp_handler_clone.send(&elevator.inn_address, &msg);
                            }
                        }
                    }
            
                    // clone elevator values from the locked mutex
                    let known_elevators_locked_clone = known_elevators_locked.clone();
                    
                    // unlock the mutex
                    drop(known_elevators_locked);

                    // get master id
                    let cloned_master_id = system_state_clone.master_id.lock().unwrap().clone();
                    
                    // skip the rest of the loop iteration if elevator is not master
                    if !(system_state_clone.me_id == cloned_master_id) {continue;} 

                    // broadcast master worldview
                    print!("BROADCASTING WORLDVIEW _____________________");
                    let worldview = make_udp_msg(system_state_clone.me_id, MessageType::Worldview, UdpData::Cabs(known_elevators_locked_clone.clone()));
                    for elevator in known_elevators_locked_clone.iter(){
                        udp_handler_clone.send(&elevator.inn_address, &worldview);
                    }

                }
        });
}

pub fn spawn_queue_finish_thread(
    system_state_clone: Arc<SystemState>, 
    udp_handler_clone: Arc<UdpHandler>, 
    elevator_clone: Elevator,
    io_channels_clone: IoChannels
) -> () {
    // clone relevant io channels
    let door_tx_clone = io_channels_clone.door_tx;
    let obstruction_tx_clone = io_channels_clone.obstruction_rx;
    spawn(move|| {
        // loop forever
        loop{
            sleep(Duration::from_secs(1));
            
            // get known elevators
            let mut known_elevators_locked = system_state_clone.known_elevators.lock().unwrap();
            // skip rest of loop iteration if the queue is empty
            if (known_elevators_locked.get_mut(0).unwrap().queue.is_empty()){continue;}

            // go to next floor
            known_elevators_locked.get_mut(0).unwrap().go_next_floor(door_tx_clone.clone(),obstruction_tx_clone.clone() ,elevator_clone.clone());
            
            // turn on lights in queue
            known_elevators_locked.get_mut(0).unwrap().turn_on_just_lights_in_queue(elevator_clone.clone());

            // print status
            known_elevators_locked.get_mut(0).unwrap().print_status();
            
            drop(known_elevators_locked);
        }
    });
}