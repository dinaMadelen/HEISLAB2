//! ## Slave Module
//! This module provides structs and functions for slave nodes
//! 
//! 
//! ## The functions includes:
//! - 'receive_order'
//! - 'notify_completed'
//! - 'cancel_order'
//! - 'update_from_worldview'
//! - 'notify_wordview_error'
//! - 'check_master_failure'
//! - 'set_new_master'
//! - 'reboot_program'
//! 
//! ## Dependencies
//! 
//! ```toml
//! [dependencies]
//! ```

//the comments are verbose so we can autogenerate documentation using 'cargo doc' https://blog.guillaume-gomez.fr/articles/2020-03-12+Guide+on+how+to+write+documentation+for+a+Rust+crate

#[warn(non_snake_case)]
#[allow(unused_imports)]
#[allow(unused_variables)]

//-----------------------IMPORTS------------------------------------------------------------

use crate::modules::cab_object::cab::Cab; //Import for cab struct
use crate::modules::udp_functions::udp::{UdpMsg, UdpData, MessageType, UdpHandler, udp_broadcast, make_udp_msg,udp_ack};
use crate::modules::order_object::order_init::Order;
use crate::modules::master_functions::master::Role;
use crate::modules::elevator_object::elevator_init::SystemState;
use crate::modules::elevator_object::alias_lib::{CAB};


use std::net::SocketAddr;
use std::thread::sleep;
use std::time::Duration; //https://doc.rust-lang.org/std/time/struct.Instant.html
use std::env; // Used for reboot function
use std::process::{Command, exit}; //Used for reboot function
use std::sync::Arc;

//-----------------------STRUCTS------------------------------------------------------------


//----------------------Fucntions-----------------------------------------------------------

/// recive_order
/// Recive order from master and add it to queue if it is not already in the queue
/// then ack the master that the order has been added
/// 
/// # Arguments:
/// 
/// * `slave` - &mut Cab - mutable refrence to the elevator where the order should be addded.
/// * `new_order` - u8 - floor that should be added to the queue.
/// * `socket` - &UdpSocket - socket of the sender.
/// * `master_adress` - SocketAddr - adress where the master is expecting the ack.
/// * `original_msg` - &UdpMsg - refrence to the udpmessage where the slave recived the order.
/// 
/// # Returns:
///
/// Returns - bool- 'true' if order has been added to queue or the order already was in the queue and ackowledgement has been sent, if the acknowledgement failed it returns 'false' 
///
pub fn receive_order(slave: &mut Cab, new_order: Order, master_address: SocketAddr, original_msg: &UdpMsg, udp_handler: &UdpHandler) -> bool {
    
    // Add order 
    if !slave.queue.contains(&new_order) {
        slave.queue.push(new_order.clone());
        println!("{} added to elevator {}", new_order.floor, slave.id);
        return udp_ack(master_address, original_msg, slave.id, udp_handler);
        // Order already exists  
    }else{
        println!("{} already in queue for elevator {}", new_order.floor, slave.id);
        return udp_ack(master_address, original_msg, slave.id, udp_handler);
    }
}

/// notify_completed
/// Broadcast that an order is completed
/// 
/// # Arguments:
/// 
/// * `completed_order` - Order - the order that was completed.
/// * `status` - &SystemState - refrence to the system state.
/// 
/// # Returns:
///
/// Returns - bool - 'true' if succsessful broadcast, 'false' if failed to broadcast.
///
pub fn notify_completed(completed_order: Order, state: &SystemState) -> bool {

    //Lock active elevators
    let known_elevators_locked = state.known_elevators.lock().unwrap();

    // Send message with order to remove
    if let Some(elevator) = known_elevators_locked.iter().find(|e| e.id == state.me_id) {
        let mut responsible_elevator = elevator.clone();
        responsible_elevator.queue = vec![completed_order.clone()];

        let message = make_udp_msg(state.me_id,MessageType::OrderComplete, UdpData::Cab(responsible_elevator.clone()));
        drop(known_elevators_locked);

        //remove it from all orders
        let mut all_orders_locked = state.all_orders.lock().unwrap();
        if let Some(index) = all_orders_locked.iter().position(|o| *o == completed_order) {
            all_orders_locked.remove(index);
        }

        return udp_broadcast(&message);

    }else{
        println!("Error:Elevator  {} is missing from active", state.me_id);
        return false;
    }
}

/// cancel_order
/// Remove an active order from a queue
/// 
/// # Arguments:
/// 
/// * `slave` - &mut Cab - mutable refrence to the elevator where the order should be removed from.
/// * `order` - u8 - order that should be removed from queue.
/// 
/// # Returns:
///
/// Returns - bool - returns 'true' if the order was successuly removed, returns 'false' if the floor couldnt be found in the queue.
///
pub fn cancel_order(slave: &mut Cab, order: Order) -> bool {

    //Remove order from queue
    if let Some(index) = slave.queue.iter().position(|o| o.floor == order.floor) {
        slave.queue.remove(index);
        println!("Order {} removed from queue of elevator {}", order.floor, slave.id);
        return true;
    }
    println!("Order {} couldnt be found in queue of elevator {}", order.floor, slave.id);
    return false;
}

/// update_from_worldview
/// Checks for discrepancies between the elevators worldview and the masters worldview
/// if there are orders in the worldview that do not exist in the queue , it updates the elevator's order queue based on a received worldview.
/// if there are missing orders in the worldview, it notifies the master that there are missing orders.
/// 
/// # Arguments:
/// 
/// * `state` - &mut SystemState - mutable refrence to the systemstate.
/// * `new_worldview` - &new_worldview) - refrence to list of elevators in the new worldview.
/// 
/// 
/// # Returns:
///
/// Returns -bool - returns 'true' if added orders or orders match, returns 'false' if there are missing orders in worldview.
///
pub fn update_from_worldview(state: &Arc<SystemState>, new_worldview: &Vec<Cab>,udp_handler: Arc<UdpHandler>) -> bool {

    let mut worldview_missing_orders = false;

    

    // Compare recived worldview to known elevators
    for wv_elevator in new_worldview{

        //Lock known elevators
        let mut known_elevators_locked = state.known_elevators.lock().unwrap();

        if let Some(elevator) = known_elevators_locked.iter_mut().find(|e| e.id == wv_elevator.id){


            let known_queue=elevator.queue.clone();

            //No new orders

            //Check if elevator is alive or dead
            if elevator.alive != wv_elevator.alive{
                worldview_missing_orders = true;
            }

            if known_queue == wv_elevator.queue{
                println!("Worldview matches for ID:{}", elevator.id);
                continue;
            }

            //Found missing order, add them to queue
            let missing_orders: Vec<Order> = wv_elevator.queue.iter().filter(|&order| !known_queue.contains(order)) .cloned().collect();
            if !missing_orders.is_empty() {
                println!("Elevator {} is missing orders {:?}. Adding...", elevator.id, missing_orders);
                elevator.queue.extend(missing_orders);
            }

        } else{
            // Add missing worldview elevator to active elevators
            println!("Found missing elevator, Adding new elevator ID {} from worldview.", wv_elevator.id);
            known_elevators_locked.push(wv_elevator.clone());
            worldview_missing_orders = true;
        }
    
        drop(known_elevators_locked);

    } 
    
    if worldview_missing_orders{

        let master_id = state.master_id.lock().unwrap().clone();
        let known_elevators_locked = state.known_elevators.lock().unwrap().clone();

        if let Some(master_elevator) = known_elevators_locked.iter().find(|e| e.id == master_id) {
            let master_address = master_elevator.inn_address.clone();
            notify_worldview_error(state.me_id,master_address,state,udp_handler);
        }
    }

    return worldview_missing_orders;
    
}

/// Missing order in worldview, notify master that there is a missing order/orders
/// 
/// # Arguments:
/// 
/// * `sender_id`  -u8- id of the sender
/// * `master_adress` - String - Adress of the master. // NEED TO FIX THIS 
/// * `missing_orders` - &Vec<Cab> - refrence to the worldview that has more orders than the new worldview 
/// * `udp_handler` - &UdpHandler - refrence to the handler that that handles the sending.
/// 
/// 
/// # Returns:
///
/// Returns - None - .
///
pub fn notify_worldview_error(sender_id: u8 ,master_adress: SocketAddr, state: &Arc<SystemState> ,udp_handler: Arc<UdpHandler>) {

    let all_cabs = state.known_elevators.lock().unwrap().clone();

    let message = make_udp_msg(sender_id,MessageType::ErrorWorldview, UdpData::Cabs(all_cabs));
    udp_handler.send(&master_adress, &message);
}


/// Check for worldview, no update in given time 5s?, assumes dead master and starts master election
/// 
/// # Arguments:
/// 
/// * `me` - &mut Eelvator - mutable refrence to the elevators.
/// * `state` - &mut SystemState - mutable refrence to the system state
/// 
/// # Returns:
///
/// Returns - bool - returns `true` if master is dead, repeats untill master is dead.
///
pub fn check_master_failure(state: &Arc<SystemState>, udp_handler: &UdpHandler) {
    
    //If i am the master, i am alive
    if state.me_id == state.master_id.lock().unwrap().clone(){
        return;
    }      

    //Get time of last lifesign
    let last_lifesign_locked = state.lifesign_master.lock().unwrap();
    

    //Check age of new lifesign
    if  last_lifesign_locked.elapsed() > Duration::from_millis(3000) {
        println!("No lifesign from master recived from Master in last 3sec, electing new master");
        //ITERATE THROUGH ELEVATORS, SET OLD MASTER TO DEAD AND RETRIEVE THE CAB STRUCT THAT WAS THE MASTER

        let master_id = state.master_id.lock().unwrap();

        //BROADCAST DEATH OF THE MASTER
        let known_elevators = state.known_elevators.lock().unwrap();
        let dead_elevator: Vec<Cab> = known_elevators.iter().filter(|cab| cab.id == *master_id).cloned().collect();
        let msg = make_udp_msg(state.me_id, MessageType::ErrorOffline, UdpData::Cab(dead_elevator.get(0).unwrap().clone()));
        for elevator in known_elevators.iter(){
            udp_handler.send(&elevator.inn_address, &msg);
        }

    }
    drop(last_lifesign_locked);

    let last_lifesign_locked = state.lifesign_master.lock().unwrap();
    if  last_lifesign_locked.elapsed() > Duration::from_millis(10000){
        let known_elevators_cloned = state.known_elevators.lock().unwrap().clone();
        set_new_master(&mut known_elevators_cloned.get(0).unwrap().clone(), &state);
    }
}


/// Wait ID*150ms before checking if the master role is taken, if not assume master role and broadcast worldview
/// 
/// # Arguments:
/// 
/// * `me` - &mut Cab - mutable refrence to this elevator.
/// * `state` - &mut SystemState - mutable refrence to the system state.
///  
/// # Returns:
///
/// Returns - None - .
///
pub fn set_new_master(new_master: &mut Cab, state: &Arc<SystemState>){
    println!("Entered set new master");

    let old_master_id = state.master_id.lock().unwrap().clone();
    let mut known_elevators_locked = state.known_elevators.lock().unwrap();

    //MAKE SET OLD MASTER SLAVE
    if let Some(old_master) = known_elevators_locked.iter_mut().find(|e| e.id == old_master_id) {
        old_master.role = Role::Slave;
        println!("Old master (ID: {}) set to Slave.", old_master.id);
    }

    //Find master id
    let mut master_id_locked = state.master_id.lock().unwrap();
    *master_id_locked =new_master.id;
    drop(master_id_locked);

    if let Some(new_master_cab) = known_elevators_locked.iter_mut().find(|e|e.id == new_master.id){
        new_master_cab.role = Role::Master;
        println!("New master (ID: {}).", new_master_cab.id);
        drop(known_elevators_locked);
    } else {
        println!("ERROR: New master is not an active elevator");
    }  
    
}

/// Starts a new instance and kills the old instance of the program
/// 
/// # Arguments:
/// 
/// * None
/// 
/// # Returns:
///
/// Returns - None - .
///
pub fn reboot_program(){
    Command::new(env::current_exe().expect("Failed to find path to program"))
        .spawn()
        .expect("Failed to restart program, Restart program manually");
    exit(0); // Kill myself
}


pub fn send_new_online(state: &Arc<SystemState>) -> bool {

    // Lock 
    let mut known_elevators_locked = state.known_elevators.lock().unwrap();

    //Find this elevator in systemstate
    if let Some(this_elevator) = known_elevators_locked.iter_mut().find(|e| e.id == state.me_id) {
        //ensure alive
        this_elevator.alive = true;
        // Create UdpMsg
        let data = UdpData::Cab(this_elevator.clone());
        let msg = make_udp_msg(this_elevator.id, MessageType::NewOnline, data);

        // Broadcast the message to notify others that this elevator is online
        return udp_broadcast(&msg);
    } 
    return false;
}

pub fn send_error_offline(state: &Arc<SystemState>) -> bool {

    // Lock 
    let mut known_elevators_locked = state.known_elevators.lock().unwrap();

    //Find this elevator in systemstate
    if let Some(my_elevator) = known_elevators_locked.iter_mut().find(|e| e.id == state.me_id) {

        //Set dead
        my_elevator.alive = false;
        // Create UdpMsg
        let data = UdpData::Cab(my_elevator.clone());
        let msg = make_udp_msg(my_elevator.id, MessageType::ErrorOffline, data);
        //Empty my queue of all orders taht are not cab
        if let Some(elevator) = known_elevators_locked.iter_mut().find(|e| e.id == state.me_id) {
            elevator.queue.retain(|o| o.order_type == CAB);
        }

        // Broadcast the message to notify others that this elevator is going offline
        return udp_broadcast(&msg);
    } else {
        println!(
            "ERROR: Elevator with ID {} not found in known_elevators. Cannot send ErrorOffline, Rebooting",
            state.me_id
        );
        // Dont know what is wrong, lets just reboot
        reboot_program();
        return false;
    }
}