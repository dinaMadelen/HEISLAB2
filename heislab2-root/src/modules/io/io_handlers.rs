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
        udp_wrapper::*,
        handlers::*,
    }
};


/// Is called when input is detected in the light rx channel
/// Turns on the lights for its own queue
pub fn handle_light_update_rx(system_state_clone: Arc<SystemState>, elevator_clone: Elevator) -> () {
    //Turn onn all lights in own queue
    let mut known_elevators_clone = system_state_clone.known_elevators.lock().unwrap().clone();
    known_elevators_clone.get_mut(0).unwrap().lights(&system_state_clone.clone(), elevator_clone);
}

/// Is called when input is detected in the order upate rx channel
/// 
pub fn handle_order_update_rx(
    system_state_clone: Arc<SystemState>,
    udphandler_clone: Arc<UdpHandler>,
    elevator_clone: Elevator,
    io_channels_clone: IoChannels
) -> () {
    let mut known_elevators_locked = system_state_clone.known_elevators.lock().unwrap();
    if known_elevators_locked.is_empty(){

    }else{
        println!("current queue: {:?}",known_elevators_locked.get_mut(0).unwrap().queue);
        let cab_clone = known_elevators_locked.get(0).unwrap().clone();
        if known_elevators_locked.get_mut(0).unwrap().status == Status::Idle {
            let imalive = make_udp_msg(system_state_clone.me_id, MessageType::ImAlive, UdpData::Cab(cab_clone));
            for elevator in known_elevators_locked.iter(){
                udphandler_clone.send(&elevator.inn_address, &imalive);
            }
            //udp_broadcast(&imalive);
        }
        known_elevators_locked.get_mut(0).unwrap().go_next_floor(io_channels_clone.door_tx,io_channels_clone.obstruction_rx, elevator_clone);
        drop(known_elevators_locked);
    }
}

pub fn handle_door_rx(system_state_clone: Arc<SystemState>,
    udphandler_clone: Arc<UdpHandler>,
    door_rx_msg: bool,
    elevator: &Elevator
) -> () {
    let door_signal = door_rx_msg;
    if door_signal {
        elevator.door_light(false);
        // LA TIL DETTE CHRIS
        let mut known_elevators_locked = system_state_clone.known_elevators.lock().unwrap().clone();
        if known_elevators_locked.get_mut(0).unwrap().queue.is_empty(){
            
        }else {
            let mut known_elevators_locked = system_state_clone.known_elevators.lock().unwrap();

            known_elevators_locked.get_mut(0).unwrap().set_status(Status::Idle, elevator.clone());
            let completed_order = known_elevators_locked.get_mut(0).unwrap().queue.remove(0);

            drop(known_elevators_locked);

            elevator.call_button_light(completed_order.floor, completed_order.order_type, false);

            let mut all_orders_locked = system_state_clone.all_orders.lock().unwrap();
            if completed_order.order_type == CAB {
                if let Some(index) = all_orders_locked.iter().position(|order| (order.floor == completed_order.floor)&& (order.order_type == CAB)) {
                    all_orders_locked.remove(index);
                }
            } else {
                all_orders_locked.retain(|order| {
                    !((order.floor == completed_order.floor )&& (order.order_type == completed_order.order_type))
                });
            }
            drop(all_orders_locked);
            
            // LA TIL DETTE CHRIS END ------  - -- -- - --  -- -- - -    -   -       -   -              -
            let known_elevators_locked = system_state_clone.known_elevators.lock().unwrap();
            let cab_clone = known_elevators_locked.get(0).unwrap().clone();

            let alive_msg = make_udp_msg(system_state_clone.me_id, MessageType::ImAlive, UdpData::Cab(cab_clone.clone()));
            let ordercomplete = make_udp_msg(system_state_clone.me_id, MessageType::OrderComplete, UdpData::Cab(cab_clone.clone()));
            drop(known_elevators_locked);

            let elevator_addresses: Vec<_> = {
                let known_elevators = system_state_clone.known_elevators.lock().unwrap();
                known_elevators.iter().map(|e| e.inn_address).collect()
            };

            for addr in elevator_addresses {
                //FJERNET NOE HER KRIS -- - -- -- - -- -- - -- - -
                udphandler_clone.send(&addr, &ordercomplete);
                udphandler_clone.send(&addr, &alive_msg); 
            }  
        }
        
    }
        
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
    {   
        //DETTE ER ENDRA _________________________________
        let new_req_msg = make_udp_msg(system_state_clone.me_id, MessageType::NewRequest, UdpData::Order(new_order.clone()));
        let known_elevators_locked = system_state_clone.known_elevators.lock().unwrap().clone();
        for elevator in known_elevators_locked.iter(){
            
            let send_successfull = udphandler_clone.send(&elevator.inn_address, &new_req_msg);

            if !send_successfull {handle_new_request(&new_req_msg,
                 Arc::clone(&system_state_clone),
                 Arc::clone(&udphandler_clone), 
                 io_channels_clone.order_update_tx.clone(), 
                 io_channels_clone.light_update_tx.clone());                    
            }
        }
        drop(known_elevators_locked);
    }

    //cab.turn_on_queue_lights(elevator.clone());
    let mut known_elevators_locked = system_state_clone.known_elevators.lock().unwrap();

    //Safety if elevator is idle to double check if its going to correct floor
    if known_elevators_locked.is_empty(){
        println!("No active elevators, not even this one ID:{}",system_state_clone.me_id);

    }else if known_elevators_locked.get_mut(0).unwrap().status == Status::Idle{
        known_elevators_locked.get_mut(0).unwrap().go_next_floor(io_channels_clone.door_tx.clone(),io_channels_clone.obstruction_rx.clone(),elevator.clone());
        if known_elevators_locked.get_mut(0).unwrap().status == Status::Moving{
            let alive_msg = make_udp_msg(system_state_clone.me_id, MessageType::ImAlive, UdpData::Cab(known_elevators_locked.get(0).unwrap().clone()));
            for elevator in known_elevators_locked.iter(){
                udphandler_clone.send(&elevator.inn_address, &alive_msg);
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
    println!("Floor: {:#?}", floor_rx_msg);
    //update current floor status

    let mut known_elevators_locked = system_state_clone.known_elevators.lock().unwrap();
    known_elevators_locked.get_mut(0).unwrap().current_floor = floor_rx_msg;
    drop(known_elevators_locked);

    //Do stuff
    let mut known_elevators_locked = system_state_clone.known_elevators.lock().unwrap();
    if known_elevators_locked.get_mut(0).unwrap().queue.is_empty(){
        elevator.motor_direction(DIRN_STOP);
    }
    known_elevators_locked.get_mut(0).unwrap().go_next_floor(io_channels_clone.door_tx.clone(),io_channels_clone.obstruction_rx.clone(),elevator.clone());

    drop(known_elevators_locked);
    let mut known_elevators_clone = system_state_clone.known_elevators.lock().unwrap().clone();
    known_elevators_clone.get_mut(0).unwrap().lights(&system_state_clone.clone(), elevator.clone());
    


    //Broadcast new state
    let  known_elevators_locked = system_state_clone.known_elevators.lock().unwrap();
    let cab_clone = known_elevators_locked.get(0).unwrap().clone();
    let alive_msg = make_udp_msg(system_state_clone.me_id, MessageType::ImAlive, UdpData::Cab(cab_clone));
        for elevator in known_elevators_locked.iter(){
            udphandler_clone.send(&elevator.inn_address, &alive_msg);
           }
    drop(known_elevators_locked);
}

pub fn handle_stop_rx(
    stop_rx_msg: bool,
    system_state_clone: Arc<SystemState>,
    elevator: & Elevator
) -> () {
    println!("Stop button: {:#?}", stop_rx_msg);
    let mut known_elevators_locked = system_state_clone.known_elevators.lock().unwrap();
    if known_elevators_locked.is_empty(){
        println!("There are no elevators in the system");
    }else {
        if known_elevators_locked.get(0).unwrap().status == Status::Stop{
            known_elevators_locked.get_mut(0).unwrap().alive=true;
            known_elevators_locked.get_mut(0).unwrap().set_status(Status::Stop, elevator.clone());
            drop(known_elevators_locked);
            let system_state_clone = Arc::clone(&system_state_clone);
            send_new_online(&system_state_clone);

        }else{
            known_elevators_locked.get_mut(0).unwrap().set_status(Status::Stop, elevator.clone());
            
            //WHO CONTROLS THE LIGHTS
            known_elevators_locked.get_mut(0).unwrap().turn_off_lights(elevator.clone());
            drop(known_elevators_locked);
            let system_state_clone = Arc::clone(&system_state_clone);
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
    //elevator.motor_direction(if obstr { DIRN_STOP } else { dirn });
    let mut known_elevators_locked = system_state_clone.known_elevators.lock().unwrap();
    if known_elevators_locked.is_empty(){

    }else {
        //Should add cab to systemstatevec and then broadcast new state of stopped
        if obstruction_rx_msg{
            known_elevators_locked.get_mut(0).unwrap().set_status(Status::Obstruction,elevator.clone());
        }else{
            known_elevators_locked.get_mut(0).unwrap().set_status(Status::Idle,elevator.clone());
            known_elevators_locked.get_mut(0).unwrap().go_next_floor(io_channels_clone.door_tx,io_channels_clone.obstruction_rx,elevator.clone());
        }
        drop(known_elevators_locked);
    }
}

