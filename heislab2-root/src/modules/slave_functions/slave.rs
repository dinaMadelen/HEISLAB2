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
pub fn notify_completed(completed_order: Order, status: &SystemState) -> bool {

    //Lock active elevators
    let active_elevators_locked = status.active_elevators.lock().unwrap();

    // Send message with order to remove
    if let Some(elevator) = active_elevators_locked.iter().find(|e| e.id == status.me_id) {
        let mut remove_elevator = elevator.clone();
        remove_elevator.queue = vec![completed_order];

        let message = make_udp_msg(status.me_id,MessageType::OrderComplete, UdpData::Cab(remove_elevator.clone()));
        return udp_broadcast(&message);

    }else{
        println!("Error:Elevator  {} is missing from active", status.me_id);
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
pub fn update_from_worldview(state: &Arc<SystemState>, new_worldview: &Vec<Cab>) -> bool {

    let mut worldview_changed = false;

    //Lock active elevators
    let mut active_elevators_locked = state.active_elevators.lock().unwrap();

    // Compare recived worldview to active elevators
    for wv_elevator in new_worldview{
        if let Some(elevator) = active_elevators_locked.iter_mut().find(|e| e.id == wv_elevator.id){


            let active_queue=elevator.queue.clone();

            //No new orders
            if active_queue == wv_elevator.queue{
                println!("Worldview matches for ID:{}", elevator.id);
                continue;
            }

            //Found missing order, add them to queue
            let missing_orders: Vec<Order> = wv_elevator.queue.iter().filter(|&order| !active_queue.contains(order)) .cloned().collect();
            if !missing_orders.is_empty() {
                println!("Elevator {} is missing orders {:?}. Adding...", elevator.id, missing_orders);
                elevator.queue.extend(missing_orders);
                worldview_changed = true;
            }

        } else{
            // Add missing worldview elevator to active elevators
            println!("Found missing elevator, Adding new elevator ID {} from worldview.", wv_elevator.id);
            active_elevators_locked.push(wv_elevator.clone());
            worldview_changed = true;
        }
    }   
    return worldview_changed;
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
pub fn notify_worldview_error(sender_id: u8 ,master_adress: String , missing_orders: &Vec<Cab>,udp_handler: &UdpHandler) {
    let message = make_udp_msg(sender_id,MessageType::ErrorWorldview, UdpData::Cabs(missing_orders.clone()));
    let socket: SocketAddr = master_adress.parse().expect("invalid adress");
    udp_handler.send(&socket, &message);
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
pub fn check_master_failure(state: &Arc<SystemState>) -> bool {


    loop{

        //Wait 5sec
        sleep(Duration::from_millis(5000));
        //Check if this is the master.
        let master_id_locked = state.master_id.lock().unwrap();
        if state.me_id == *master_id_locked{
            //i am the master, no need to continue checking
            return false
        }
        drop(master_id_locked);
    


        //Get time of last lifesign
        let last_lifesign_locked = state.last_lifesign.lock().unwrap();
        let last_lifesign = last_lifesign_locked.clone();
        drop(last_lifesign_locked);

        //Check age of new lifesign
        if  last_lifesign.elapsed() > Duration::from_millis(5000) {
            println!("No worldview recived from Master in last 5sec, electing new master");
            let mut active_elevators_locked = state.active_elevators.lock().unwrap();
            set_new_master(active_elevators_locked.get_mut(0).unwrap(),state)
        }
        
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
pub fn set_new_master(me: &mut Cab, state: &Arc<SystemState>){

    sleep(Duration::from_millis(150*u64::from(me.id)));
    println!("Entered set new master");
    let last_lifesign_locked = state.last_lifesign.lock().unwrap();
    if last_lifesign_locked.elapsed() > Duration::from_millis(5000){
        //Set myself as master
    
        //Find master id
        let master_id_locked = state.master_id.lock().unwrap();
        let master_id=master_id_locked.clone();
        drop(master_id_locked);
        
        //This causes deadlock since calling the cab already requires locking the mutex
        let mut active_elevators_locked = state.active_elevators.lock().unwrap();
        if let Some(old_master) = active_elevators_locked.iter_mut().find(|e| e.id == master_id) {
            old_master.role = Role::Slave;
            println!("Old master (ID: {}) set to Slave.", old_master.id);
        }
        if let Some(new_master) = active_elevators_locked.iter_mut().find(|e|e.id == state.me_id){
            new_master.role = Role::Master;
            {
                let mut master_id_locked = state.master_id.lock().unwrap();
                *master_id_locked = state.me_id;
            }
            
            println!("New master (ID: {}).", new_master.id);
            let message = make_udp_msg(me.id,MessageType::NewMaster, UdpData::Cab(new_master.clone()));
            udp_broadcast(&message);
            drop(active_elevators_locked);
        } else {
            println!("ERROR: New master is not an active elevator");
        }
        
    }else{
        
        //Someone sendt worldview, and became the new master
        let last_worldview_locked=state.last_worldview.lock().unwrap();
        let last_worldview=last_worldview_locked.clone();
        drop(last_worldview_locked);
        let mut master_id_locked=state.master_id.lock().unwrap();
        *master_id_locked=last_worldview.header.sender_id;
        drop(master_id_locked)
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


pub fn send_new_online(state: &SystemState) -> bool {
    // Lock 
    let active_elevators_locked = state.active_elevators.lock().unwrap();

    //Find this elevator in systemstate
    if let Some(this_elevator) = active_elevators_locked.iter().find(|e| e.id == state.me_id) {
        // Create UdpMsg
        let data = UdpData::Cab(this_elevator.clone());
        let msg = make_udp_msg(this_elevator.id, MessageType::NewOnline, data);

        // Broadcast the message to notify others that this elevator is online
        return udp_broadcast(&msg);
    } 
    return false;
}

pub fn send_error_offline(state: &SystemState) -> bool {
    // Lock 
    let active_elevators = state.active_elevators.lock().unwrap();

    //Find this elevator in systemstate
    if let Some(my_elevator) = active_elevators.iter().find(|e| e.id == state.me_id) {
        // Create UdpMsg
        let data = UdpData::Cab(my_elevator.clone());
        let msg = make_udp_msg(my_elevator.id, MessageType::ErrorOffline, data);

        // Broadcast the message to notify others that this elevator is going offline
        return udp_broadcast(&msg);
    } else {
        println!(
            "ERROR: Elevator with ID {} not found in active_elevators. Cannot send ErrorOffline.",
            state.me_id
        );
        return false;
    }
}