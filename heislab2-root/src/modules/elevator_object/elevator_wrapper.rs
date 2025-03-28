use std::time::*;
use std::sync::Arc;
use std::thread::*;

use crate::modules::{
    elevator_object::elevator_init::Elevator, 
    io::io_init::IoChannels, 
    udp_functions::udp::*,
    master_functions::master::*,
    cab_object::elevator_status_functions::*,
    slave_functions::slave::*,
};
use super::elevator_init::SystemState;

/// Goes down until a floor is found
pub fn go_down_until_floor_found(elevator: &Elevator, dirn: u8) -> () {
    if elevator.floor_sensor().is_none() {
        elevator.motor_direction(dirn);
    }
}

pub fn spawn_elevator_monitor_thread(system_state_clone: Arc<SystemState>, udp_handler_clone: Arc<UdpHandler>) -> () {
    // threshold for when elevator is considered dead
    const DEAD_ELEV_THRESHOLD:Duration = Duration::from_secs(10);
        spawn(move||{
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
            sleep(Duration::from_millis(2000));   
            loop{
                fix_master_issues(&system_state_clone, &udp_handler_clone);

                sleep(Duration::from_millis(400));
                let now = SystemTime::now();
                
                // Iterate in reverse order so that removing elements doesn't affect things
                {
                    let  known_elevators_locked = system_state_clone.known_elevators.lock().unwrap();
                    for i in (0..known_elevators_locked.len()).rev() {
                        let elevator = &known_elevators_locked[i];
                        // Only check elevators that are Moving or DoorOpen.
                        if elevator.status == Status::Moving || elevator.status == Status::DoorOpen {
                            if let Ok(elapsed) = now.duration_since(elevator.last_lifesign) {
                                if elapsed >= DEAD_ELEV_THRESHOLD {
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
                sleep(Duration::from_millis(400));
                check_master_failure(&system_state_clone, &udp_handler_clone);
            }
        });
}

pub fn spawn_queue_finish_thread(
    system_state_clone: Arc<SystemState>, 
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
            if known_elevators_locked.get_mut(0).unwrap().queue.is_empty() {continue;}

            // go to next floor
            known_elevators_locked.get_mut(0).unwrap().go_next_floor(door_tx_clone.clone(),obstruction_tx_clone.clone() ,elevator_clone.clone());
            
            // turn on lights in queue
            let queue_clone = known_elevators_locked.get_mut(0).unwrap().queue.clone();
            known_elevators_locked.get_mut(0).unwrap().lights(queue_clone, elevator_clone.clone());

            // print status
            known_elevators_locked.get_mut(0).unwrap().print_status();
            
            drop(known_elevators_locked);
        }
    });
}

pub fn initialize_elevator(port: i32, elev_num_floors: u8) -> Elevator {
    let elev_server_port = format!("localhost:{}", port);
    let elevator = Elevator::init(&elev_server_port, elev_num_floors).unwrap();
    println!("Elevator started:\n{:#?}", elevator);
    elevator
}
