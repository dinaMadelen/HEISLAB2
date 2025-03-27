//---------
// Imports
//---------
// public crates
use std::thread::*;
use std::time::*;
use crossbeam_channel as cbc;
use std::sync::Arc;
use std::net::{SocketAddr, IpAddr, Ipv4Addr};

// our crates
use heislab2_root::modules::elevator_object::*;
use alias_lib::DIRN_DOWN;
use elevator_init::Elevator;
use heislab2_root::modules::*;

use cab_object::*;
use cab::Cab;
use elevator_status_functions::Status;
use order_object::order_init::Order;
use slave_functions::slave::*;
use master_functions::master::*;
use system_init::*;

use heislab2_root::modules::udp_functions::udp::*;
use udp_functions::udp::UdpData;
use udp_functions::udp_wrapper;
use heislab2_root::modules::io::io_init::*;
use heislab2_root::modules::cab_object::cab_wrapper;

//------
// Main
//------
fn main() -> std::io::Result<()> {
    //----------------
    // Initialization
    //----------------

    // create elevator
    let elev_num_floors = 4;
    let elevator = Elevator::init("localhost:15657", elev_num_floors)?;
    println!("Elevator started:\n{:#?}", elevator);

    // create empty worldview dummy message  
    let boot_worldview = udp_wrapper::create_empty_worldview_msg();

    // initialize cab
    let system_state = initialize_system_state();
    let mut cab = cab_wrapper::initialize_cab(elev_num_floors, &system_state, elevator.clone())?;

    // initialize udp handler
    let udphandler = Arc::new(init_udp_handler(cab.clone()));

    //Lock free cab into captivity:(
    let mut active_elevators_locked = system_state.active_elevators.lock().unwrap();
    active_elevators_locked.push(cab);
    drop(active_elevators_locked);

    println!("Cab initialized:\n{:#?}", elevator);

    // --------------INIT CAB FINISH---------------
    
    // --------------INIT CHANNELS---------------
    let io_channels = IoChannels::new(&elevator);
    // --------------INIT CHANNELS FINISHED---------------

    // --------------INIT RECIEVER THREAD------------------
    let system_state_clone = Arc::clone(&system_state);
    
    // -------------------SET MASTER ID------------------
    let mut active_elevators_locked = system_state.active_elevators.lock().unwrap();
    let mut cab_clone = active_elevators_locked.get_mut(0).unwrap().clone();
    drop(active_elevators_locked);


    set_new_master(&mut cab_clone, &system_state);
    // -------------------SET MASTER ID FINISHED------------------

    let udphandler_clone = Arc::clone(&udphandler);
    spawn(move||{
        loop{

            let handler = Arc::clone(&udphandler_clone); 
            handler.receive(60000, &system_state_clone, io_channels.order_update_tx.clone(), io_channels.light_update_tx.clone());
        }
    });
    // -------------INIT RECIEVER FINISHED-----------------

    //ELEVATORMONITOR!!!
    let system_state_clone = Arc::clone(&system_state);
    let elevator_clone = elevator.clone();
    spawn(move||{
            loop{
                // Sleep for 5 seconds between checks.
                sleep(Duration::from_secs(1));
                let mut dead_elevators_locked = system_state_clone.dead_elevators.lock().unwrap();
                
                for cab in dead_elevators_locked.iter_mut(){
                    cab.turn_off_lights_not_in_queue(elevator_clone.clone());
                };

                drop(dead_elevators_locked);
                sleep(Duration::from_secs(4));
                let now = SystemTime::now();

                // Lock active_elevators.
                let mut active_elevators_locked = system_state_clone.active_elevators.lock().unwrap();
                // Iterate in reverse order so that removing elements doesn't affect our indices.
                for i in (0..active_elevators_locked.len()).rev() {
                    let elevator = &active_elevators_locked[i];
                    // Only check elevators that are Moving or DoorOpen.
                    if elevator.status == Status::Moving || elevator.status == Status::DoorOpen {
                        if let Ok(elapsed) = now.duration_since(elevator.last_lifesign) {
                            if elapsed >= Duration::from_secs(5) {
                                // Remove the elevator from active list. and add to dead list
                                //Before adding to dead list remove all but cab calls from the dead elevators queue 
                                let dead_elevator = active_elevators_locked.remove(i);
                                let mut dead_elevators_locked = system_state_clone.dead_elevators.lock().unwrap();
                                dead_elevators_locked.push(dead_elevator.clone());
                                drop(dead_elevators_locked);
                                println!("Elevator {} is dead (elapsed: {:?})", dead_elevator.id, elapsed);
                                let msg = make_udp_msg(system_state_clone.me_id, MessageType::ErrorOffline, UdpData::Cab(dead_elevator));
                                udp_broadcast(&msg);
                            }
                        }
                    }
                }

                {   
                    let locked_master_id = system_state_clone.master_id.lock().unwrap();
                    if system_state_clone.me_id == *locked_master_id{
                        //let msg = make_udp_msg(system_state_clone.me_id, MessageType::Worldview, UdpData::Cabs(active_elevators_locked.clone()));
                        //udp_broadcast(&msg);
                        master_worldview(&system_state_clone);
                    }
                }
            }
    });

    //INIT OVER
    //TEST IF MASTER FAILURE WORKS!!!!!
    /* 
    let system_state_clone = Arc::clone(&system_state);
    
    spawn(move||{
        check_master_failure(&system_state_clone);
    });
    */

    let dirn = DIRN_DOWN;
    if elevator.floor_sensor().is_none() {
        elevator.motor_direction(dirn);
    }

    let  active_elevators_locked = system_state.active_elevators.lock().unwrap();
    let cab_clone = active_elevators_locked.get(0).unwrap().clone();
    drop(active_elevators_locked);
    
    let master_id_clone = system_state.master_id.lock().unwrap().clone();
    println!("The master is assigned as: {}",master_id_clone);

    let msg = make_udp_msg(system_state.me_id, MessageType::NewOnline, UdpData::Cab(cab_clone));
    udp_broadcast(&msg);

    // ------------------ MAIN LOOP ---------------------
    loop {
        cbc::select! {
            /* 
            recv(world_view_update_rx) -> a => {
                let world_view = a.unwrap();
                //Add to own wv then ack
                
                let msg = make_udp_msg(cab.id, MessageType::Ack, UdpData::None);
                udp_broadcast(&msg);
            },
            */
            recv(io_channels.light_update_rx) -> a => {
                let lights_to_turn_on = a.unwrap();
                //Turn onn all lights in own queue
                let mut active_elevators_locked = system_state.active_elevators.lock().unwrap();
                if lights_to_turn_on == active_elevators_locked.get_mut(0).unwrap().queue{
                    active_elevators_locked.get_mut(0).unwrap().turn_on_queue_lights(elevator.clone());
                }else{
                    active_elevators_locked.get_mut(0).unwrap().turn_off_differing_lights(elevator.clone(), lights_to_turn_on);
                }
                drop(active_elevators_locked);
            },
            recv(io_channels.order_update_rx) -> a => {

                //ASSUME THE ORDER ALREADY IS ADDED TO QUEUE
                //Mulig denne er for tidlig
                let mut active_elevators_locked = system_state.active_elevators.lock().unwrap();
                if active_elevators_locked.is_empty(){

                }else{
                    println!("current queue: {:?}",active_elevators_locked.get_mut(0).unwrap().queue);
                    let cab_clone = active_elevators_locked.get(0).unwrap().clone();
                    if active_elevators_locked.get_mut(0).unwrap().status == Status::Idle {
                        let imalive = make_udp_msg(system_state.me_id, MessageType::ImAlive, UdpData::Cab(cab_clone));
                        udp_broadcast(&imalive);
                    }

                    active_elevators_locked.get_mut(0).unwrap().go_next_floor(io_channels.door_tx.clone(),io_channels.obstruction_rx.clone(),elevator.clone());
                    drop(active_elevators_locked);
                }
            },
            
            recv(io_channels.door_rx) -> a => {
                let door_signal = a.unwrap();
                if door_signal {
                    let mut active_elevators_locked = system_state.active_elevators.lock().unwrap();
                    active_elevators_locked.get_mut(0).unwrap().set_status(Status::DoorOpen,elevator.clone());
                    active_elevators_locked.get_mut(0).unwrap().go_next_floor(io_channels.door_tx.clone(),io_channels.obstruction_rx.clone(),elevator.clone());
                    let cab_clone = active_elevators_locked.get(0).unwrap().clone();
                    drop(active_elevators_locked);
                    elevator.door_light(false);

                    let msg = make_udp_msg(system_state.me_id, MessageType::ImAlive, UdpData::Cab(cab_clone));
                    udp_broadcast(&msg);                   
                    
                }
            },

            recv(io_channels.call_rx) -> a => {
                let call_button = a.unwrap();
                println!("{:#?}", call_button);
                //Make new order and add that order to elevators queue
                //broadcast addition, but since order is in own cab the others taking over will not help
                let new_order = Order::init(call_button.floor, call_button.call);
                {   
                    //Broadcast new request
                    let msg = make_udp_msg(system_state.me_id, MessageType::NewRequest, UdpData::Order(new_order.clone()));
                    udp_broadcast(&msg);

                }

                //cab.turn_on_queue_lights(elevator.clone());
                let mut active_elevators_locked = system_state.active_elevators.lock().unwrap();
                
                //Safety if elevator is idle to double check if its going to correct floor
                if active_elevators_locked.is_empty(){
                    println!("No active elevators, not even this one ID:{}",system_state.me_id);

                }else if active_elevators_locked.get_mut(0).unwrap().status == Status::Idle{
                    println!("GOING NEXT FLOOR!");
                    active_elevators_locked.get_mut(0).unwrap().go_next_floor(io_channels.door_tx.clone(),io_channels.obstruction_rx.clone(),elevator.clone());

                    if active_elevators_locked.get_mut(0).unwrap().status == Status::Moving{
                        let msg = make_udp_msg(system_state.me_id, MessageType::ImAlive, UdpData::Cab(active_elevators_locked.get_mut(0).unwrap().clone()));
                        udp_broadcast(&msg);
                    }
                } 
                drop(active_elevators_locked);
            },

            recv(io_channels.floor_rx) -> a => {
                let floor = a.unwrap();
                println!("Floor: {:#?}", floor);
                //update current floor status

                let mut active_elevators_locked = system_state.active_elevators.lock().unwrap();
                active_elevators_locked.get_mut(0).unwrap().current_floor = floor;
                drop(active_elevators_locked);

                //Do stuff
                let mut active_elevators_locked = system_state.active_elevators.lock().unwrap();
                active_elevators_locked.get_mut(0).unwrap().go_next_floor(io_channels.door_tx.clone(),io_channels.obstruction_rx.clone(),elevator.clone());
                drop(active_elevators_locked);

                //Broadcast new state
                let  active_elevators_locked = system_state.active_elevators.lock().unwrap();
                let cab_clone = active_elevators_locked.get(0).unwrap().clone();
                drop(active_elevators_locked);

                let msg = make_udp_msg(system_state.me_id, MessageType::ImAlive, UdpData::Cab(cab_clone));
                udp_broadcast(&msg);
            },

            /*Burde nok modifiseres*/
            recv(io_channels.stop_rx) -> a => {
                let stop = a.unwrap();
                println!("Stop button: {:#?}", stop);
                let mut active_elevators_locked = system_state.active_elevators.lock().unwrap();
                if active_elevators_locked.is_empty(){

                }else {
                    active_elevators_locked.get_mut(0).unwrap().set_status(Status::Stop, elevator.clone());
                    drop(active_elevators_locked);

                    //Should add cab to systemstatevec and then broadcast new state of stopped
                    let  active_elevators_locked = system_state.active_elevators.lock().unwrap();
                    let cab_clone = active_elevators_locked.get(0).unwrap().clone();
                    drop(active_elevators_locked);

                    //WHO CONTROLS THE LIGHTS
                    let mut active_elevators_locked = system_state.active_elevators.lock().unwrap();
                    active_elevators_locked.get_mut(0).unwrap().turn_off_lights(elevator.clone());
                    drop(active_elevators_locked);
                    //broadcast current floor, stop and current queue - this might be redistributed
                }
                
            },

            recv(io_channels.obstruction_rx) -> a => {
                let obstr = a.unwrap();
                println!("Obstruction: {:#?}", obstr);
                //elevator.motor_direction(if obstr { DIRN_STOP } else { dirn });
                let mut active_elevators_locked = system_state.active_elevators.lock().unwrap();
                if active_elevators_locked.is_empty(){

                }else {
                    //Should add cab to systemstatevec and then broadcast new state of stopped
                    if obstr{
                        active_elevators_locked.get_mut(0).unwrap().set_status(Status::Obstruction,elevator.clone());
                    }else{
                        active_elevators_locked.get_mut(0).unwrap().set_status(Status::Idle,elevator.clone());
                    }
                    drop(active_elevators_locked);
                }
            },
            
        }
    }
}
