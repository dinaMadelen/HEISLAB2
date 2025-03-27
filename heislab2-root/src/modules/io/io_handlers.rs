use std::{
    thread::*,
    time::*,
    sync::Arc,
    net::{SocketAddr, IpAddr, Ipv4Addr}
};

use crossbeam_channel::{self as cbc, RecvError};

use crate::modules::{
    cab_object::{
        cab_wrapper::*,
        elevator_status_functions::Status,
    }, elevator_object::{
        alias_lib::DIRN_DOWN,
        elevator_init::Elevator,
        elevator_wrapper::*,
        poll::CallButton
    }, io::io_init::*, master_functions::{
        master::*,
        master_wrapper::*
    }, order_object::order_init::Order, slave_functions::slave::*, system_init::*, system_status::{self, SystemState}, udp_functions::{
        udp::*, 
        udp_wrapper::*
    }
};


/// Is called when input is detected in the light rx channel
/// Turns on the lights for its own queue
pub fn handle_light_update_rx(system_state_clone: Arc<SystemState>, elevator_clone: Elevator) -> () {
    // get elevators
    let mut known_elevators_locked = system_state_clone.known_elevators.lock().unwrap();
    // turn on light in own queue
    known_elevators_locked.get_mut(0).unwrap().turn_on_just_lights_in_queue(elevator_clone);
}

/// Is called when input is detected in the order upate rx channel
/// 
pub fn handle_order_update_rx(
    system_state_clone: Arc<SystemState>,
    udphandler_clone: Arc<UdpHandler>,
    elevator_clone: Elevator,
    io_channels_clone: IoChannels
) -> () {
    // get elevators
    let mut known_elevators_locked = system_state_clone.known_elevators.lock().unwrap();
    
    // if no elevators, return from function
    if known_elevators_locked.is_empty(){return}
    
    // print current queue
    println!("current queue: {:?}",known_elevators_locked.get_mut(0).unwrap().queue);

    // clone a cab
    let cab_clone = known_elevators_locked.get(0).unwrap().clone();

    // only exectute if elevator is idle
    if known_elevators_locked.get_mut(0).unwrap().status == Status::Idle {
        // craft "i am alive message"
        let imalive = make_udp_msg(system_state_clone.me_id, MessageType::ImAlive, UdpData::Cab(cab_clone));

        // send to all active elevators
        for elevator in known_elevators_locked.iter(){
            udphandler_clone.send(&elevator.inn_address, &imalive);
        }
    }

    // go to next floor
    known_elevators_locked.get_mut(0).unwrap().go_next_floor(io_channels_clone.door_tx,io_channels_clone.obstruction_rx,elevator_clone);
    drop(known_elevators_locked);
}

pub fn handle_door_rx(system_state_clone: Arc<SystemState>,
    udphandler_clone: Arc<UdpHandler>,
    io_channels_clone: IoChannels,
    door_rx_msg: bool,
    elevator: &Elevator
) -> () {
    // return from function if not door signal
    if !door_rx_msg {return};
    
    // turn of door light
    elevator.door_light(false);
    // get known elevators
    let mut known_elevators_locked = system_state_clone.known_elevators.lock().unwrap();
    // set own status to idle
    known_elevators_locked.get_mut(0).unwrap().set_status(Status::Idle, elevator.clone());
    // clone self
    let cab_clone = known_elevators_locked.get(0).unwrap().clone();

    // only execute if queue isn't empty and the current floor is the same as next in queue
    if (!cab_clone.queue.is_empty()) && (cab_clone.current_floor == (cab_clone.queue.get(0)).unwrap().floor){
        // pop floor from queue
        known_elevators_locked.get_mut(0).unwrap().queue.remove(0);
    }
    // craft order complete message
    let ordercomplete = make_udp_msg(system_state_clone.me_id, MessageType::OrderComplete, UdpData::Cab(cab_clone.clone()));

    // only execute if queue is empty
    if cab_clone.queue.is_empty(){
        println!("No orders in this elevators queue");
    }

    drop(known_elevators_locked);

    // craft i am alive message
    let msg = make_udp_msg(system_state_clone.me_id, MessageType::ImAlive, UdpData::Cab(cab_clone));
    let mut known_elevators_locked = system_state_clone.known_elevators.lock().unwrap();
        for elevator in known_elevators_locked.iter(){
            udphandler_clone.send(&elevator.inn_address, &ordercomplete);
            udphandler_clone.send(&elevator.inn_address, &msg);
        }
    known_elevators_locked.get_mut(0).unwrap().go_next_floor(io_channels_clone.door_tx,io_channels_clone.obstruction_rx, elevator.clone());
    drop(known_elevators_locked);
        
}

pub fn handle_call_rx(
    call_button_rx_msg: CallButton,
    system_state_clone: Arc<SystemState>,
    udphandler_clone: Arc<UdpHandler>,
    io_channels_clone: IoChannels,
    elevator: & Elevator
) -> () {
    println!("{:#?}", call_button_rx_msg);
    //Make new order and add that order to elevators queue
    let new_order = Order::init(call_button_rx_msg.floor, call_button_rx_msg.call);
    //Broadcast new request
    let msg = make_udp_msg(system_state_clone.me_id, MessageType::NewRequest, UdpData::Order(new_order.clone()));
    let known_elevators_locked = system_state_clone.known_elevators.lock().unwrap();
        for elevator in known_elevators_locked.iter(){
            udphandler_clone.send(&elevator.inn_address, &msg);
        }
    drop(known_elevators_locked);
       


    //cab.turn_on_queue_lights(elevator.clone());
    let mut known_elevators_locked = system_state_clone.known_elevators.lock().unwrap();

    //Safety if elevator is idle to double check if its going to correct floor
    if known_elevators_locked.is_empty(){
        println!("No active elevators, not even this one ID:{}",system_state_clone.me_id);

    }else if known_elevators_locked.get_mut(0).unwrap().status == Status::Idle{
        known_elevators_locked.get_mut(0).unwrap().go_next_floor(io_channels_clone.door_tx,io_channels_clone.obstruction_rx,elevator.clone());
        if known_elevators_locked.get_mut(0).unwrap().status == Status::Moving{
            let msg = make_udp_msg(system_state_clone.me_id, MessageType::ImAlive, UdpData::Cab(known_elevators_locked.get(0).unwrap().clone()));
            for elevator in known_elevators_locked.iter(){
                udphandler_clone.send(&elevator.inn_address, &msg);
            }
        }
    }
    drop(known_elevators_locked);
}

pub fn handle_floor_rx(
    floor_rx_msg: u8,
    system_state_clone: Arc<SystemState>,
    udphandler_clone: Arc<UdpHandler>,
    io_channels_clone: IoChannels,
    elevator: & Elevator
) -> () {
    println!("floor_rx_msg: {:#?}", floor_rx_msg);

    let mut known_elevators_locked = system_state_clone.known_elevators.lock().unwrap();
    known_elevators_locked.get_mut(0).unwrap().current_floor = floor_rx_msg;
    drop(known_elevators_locked);

    //Do stuff
    let mut known_elevators_locked = system_state_clone.known_elevators.lock().unwrap();
    known_elevators_locked.get_mut(0).unwrap().go_next_floor(io_channels_clone.door_tx.clone(),io_channels_clone.obstruction_rx.clone(),elevator.clone());
    known_elevators_locked.get_mut(0).unwrap().turn_on_just_lights_in_queue(elevator.clone());
    drop(known_elevators_locked);


    //Broadcast new state
    let  known_elevators_locked = system_state_clone.known_elevators.lock().unwrap();
    let cab_clone = known_elevators_locked.get(0).unwrap().clone();
    let msg = make_udp_msg(system_state_clone.me_id, MessageType::ImAlive, UdpData::Cab(cab_clone));
        for elevator in known_elevators_locked.iter(){
            udphandler_clone.send(&elevator.inn_address, &msg);
           }
    drop(known_elevators_locked);
}

// pub fn handle_stop_rx(
//     stop_rx_msg: bool,
//     system_state_clone: Arc<SystemState>,
//     elevator: & Elevator
// ) -> () {
//     println!("Stop button: {:#?}", stop_rx_msg);
//     let mut known_elevators_locked = system_state_clone.known_elevators.lock().unwrap();
//     if known_elevators_locked.is_empty(){
//         println!("There are no elevators in the system")
//     }else {
//         if known_elevators_locked.get(0).unwrap().status == Status::Stop{
//             known_elevators_locked.get_mut(0).unwrap().alive=true;
//             known_elevators_locked.get_mut(0).unwrap().set_status(Status::Idle, elevator.clone());
//             drop(known_elevators_locked);
//             // let mut system_state_clone = Arc::clone(&system_state_clone);
//             send_new_online(&system_state_clone);
//         }else{
//             known_elevators_locked.get_mut(0).unwrap().set_status(Status::Stop, elevator.clone());
//             //WHO CONTROLS THE LIGHTS
//             let mut known_elevators_locked = system_state_clone.known_elevators.lock().unwrap();
//             known_elevators_locked.get_mut(0).unwrap().turn_off_lights(elevator.clone());
//             drop(known_elevators_locked);
//             // let mut system_state_clone = Arc::clone(&system_state_clone);
//             send_error_offline(&system_state_clone);
//         }
//     }
// }

pub fn handle_stop_rx(
    stop_rx_msg: bool,
    system_state_clone: Arc<SystemState>,
    elevator: & Elevator
) -> () {
    println!("stop_rx_msg button: {:#?}", stop_rx_msg);
    let mut known_elevators_locked = system_state_clone.known_elevators.lock().unwrap();
    if known_elevators_locked.is_empty(){
        println!("There are no elevators in the system")
    }else {
        if known_elevators_locked.get(0).unwrap().status == Status::Stop{
            known_elevators_locked.get_mut(0).unwrap().alive=true;
            known_elevators_locked.get_mut(0).unwrap().set_status(Status::Idle, elevator.clone());
            drop(known_elevators_locked);
            let mut system_state_clone = Arc::clone(&system_state_clone);
            send_new_online(&system_state_clone);

        }else{
            known_elevators_locked.get_mut(0).unwrap().set_status(Status::Stop, elevator.clone());
            
            //WHO CONTROLS THE LIGHTS
            let mut known_elevators_locked = system_state_clone.known_elevators.lock().unwrap();
            known_elevators_locked.get_mut(0).unwrap().turn_off_lights(elevator.clone());
            drop(known_elevators_locked);
            let mut system_state_clone = Arc::clone(&system_state_clone);
            send_error_offline(&system_state_clone);
        }
    }
}

pub fn handle_obstruction_rx(
    obstruction_rx_msg: bool,
    system_state_clone: Arc<SystemState>,
    io_channels_clone: IoChannels,
    elevator: & Elevator
) -> () {
    println!("Obstruction: {:#?}", obstruction_rx_msg);
    //elevator.motor_direction(if obstruction_rx_msg { DIRN_STOP } else { dirn });
    let mut known_elevators_locked = system_state_clone.known_elevators.lock().unwrap();
    if known_elevators_locked.is_empty(){

    }else {
        //Should add cab to systemstatevec and then broadcast new state of stopped
        if obstruction_rx_msg{
            known_elevators_locked.get_mut(0).unwrap().set_status(Status::Obstruction,elevator.clone());
        }else{
            known_elevators_locked.get_mut(0).unwrap().set_status(Status::Idle,elevator.clone());
            known_elevators_locked.get_mut(0).unwrap().go_next_floor(io_channels_clone.door_tx.clone(),io_channels_clone.obstruction_rx.clone(),elevator.clone());
            known_elevators_locked.get_mut(0).unwrap().turn_on_just_lights_in_queue(elevator.clone());
        }
        drop(known_elevators_locked);
    }
}

