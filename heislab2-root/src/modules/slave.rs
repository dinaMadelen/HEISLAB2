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

use crate::modules::elevator_object::elevator_init::Elevator; //Import for elevator struct
use crate::modules::udp::{UdpMsg, MessageType, UdpHandler, udp_broadcast, make_Udp_msg,udp_ack};
use crate::modules::order_object::order_init::Order;
use crate::modules::master::Role;
use crate::modules::system_status::SystemState;

use std::net::{UdpSocket, SocketAddr};
use std::thread::sleep;
use std::time::{Duration}; //https://doc.rust-lang.org/std/time/struct.Instant.html
use std::env; // Used for reboot function
use std::process::{Command, exit}; //Used for reboot function


//-----------------------STRUCTS------------------------------------------------------------


//----------------------Fucntions-----------------------------------------------------------

/// recive_order
/// Recive order from master and add it to queue if it is not already in the queue
/// then ack the master that the order has been added
/// 
/// # Arguments:
/// 
/// * `slave` - &mut Elevator - mutable refrence to the elevator where the order should be addded.
/// * `new_order` - u8 - floor that should be added to the queue.
/// * `socket` - &UdpSocket - socket of the sender.
/// * `master_adress` - SocketAddr - adress where the master is expecting the ack.
/// * `original_msg` - &UdpMsg - refrence to the udpmessage where the slave recived the order.
/// 
/// # Returns:
///
/// Returns - bool- 'true' if order has been added to queue or the order already was in the queue and ackowledgement has been sent, if the acknowledgement failed it returns 'false' 
///
pub fn receive_order(slave: &mut Elevator, new_order: Order, master_address: SocketAddr, original_msg: &UdpMsg, udp_handler: &UdpHandler) -> bool {
    
    // Add order 
    if !slave.queue.contains(&new_order) {
        slave.queue.push(new_order.clone());
        println!("{} added to elevator {}", new_order.floor, slave.ID);
        return udp_ack(master_address, original_msg, slave.ID, udp_handler);
        // Order already exists  
    }else{
        println!("{} already in queue for elevator {}", new_order.floor, slave.ID);
        return udp_ack(master_address, original_msg, slave.ID, udp_handler);
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
    if let Some(elevator) = active_elevators_locked.iter().find(|e| e.ID == status.me_ID) {
        let mut remove_elevator = elevator.clone();
        remove_elevator.queue = vec![completed_order];

        let message = make_Udp_msg(MessageType::Order_Complete, &vec![remove_elevator]);
        return udp_broadcast(&message);
    }else{
        println!("Error:Elevator  {} is missing from active", status.me_ID);
        return false;
    }
}

/// cancel_order
/// Remove an active order from a queue
/// 
/// # Arguments:
/// 
/// * `slave` - &mut Elevator - mutable refrence to the elevator where the order should be removed from.
/// * `order` - u8 - order that should be removed from queue.
/// 
/// # Returns:
///
/// Returns - bool - returns 'true' if the order was successuly removed, returns 'false' if the floor couldnt be found in the queue.
///
pub fn cancel_order(slave: &mut Elevator, order: Order) -> bool {

    //Remove order from queue
    if let Some(index) = slave.queue.iter().position(|o| o.floor == order.floor) {
        slave.queue.remove(index);
        println!("Order {} removed from queue of elevator {}", order.floor, slave.ID);
        return true;
    }
    println!("Order {} couldnt be found in queue of elevator {}", order.floor, slave.ID);
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
pub fn update_from_worldview(state: &mut SystemState, new_worldview: &Vec<Elevator>) -> bool {

    let mut worldview_changed = false;

    //Lock active elevators
    let mut active_elevators_locked = state.active_elevators.lock().unwrap();

    // Compare recived worldview to active elevators
    for wv_elevator in new_worldview{
        if let Some(elevator) = active_elevators_locked.iter_mut().find(|e| e.ID == wv_elevator.ID){


            let active_queue=elevator.queue.clone();

            //No new orders
            if active_queue == wv_elevator.queue{
                println!("Worldview matches for ID:{}", elevator.ID);
                continue;
            }

            //Found missing order, add them to queue
            let missing_orders: Vec<Order> = wv_elevator.queue.iter().filter(|&order| !active_queue.contains(order)) .cloned().collect();
            if !missing_orders.is_empty() {
                println!("Elevator {} is missing orders {:?}. Adding...", elevator.ID, missing_orders);
                elevator.queue.extend(missing_orders);
                worldview_changed = true;
            }

        } else{
            // Add missing worldview elevator to active elevators
            println!("Found missing elevator, Adding new elevator ID {} from worldview.", wv_elevator.ID);
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
/// * `master_adress` - String - Adress of the master. // NEED TO FIX THIS 
/// * `missing_orders` - &Vec<Elevator> - refrence to the worldview that has more orders than the new worldview 
/// * `udp_handler` - &UdpHandler - refrence to the handler that that handles the sending.
/// 
/// 
/// # Returns:
///
/// Returns - None - .
///
pub fn notify_worldview_error(master_adress: String , missing_orders: &Vec<Elevator>,udp_handler: &UdpHandler) {

    let message = make_Udp_msg(MessageType::Error_Worldview, missing_orders);
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
pub fn check_master_failure(me:&mut Elevator, state: &mut SystemState) -> bool {

    loop{
        //Wait 5sec
        sleep(Duration::from_millis(5000));

        //Lock and check for new life sign
        let mut last_lifesign_locked = state.last_lifesign.lock().unwrap();
        if  last_lifesign_locked.elapsed() > Duration::from_millis(5000) {
            println!("No worldview recived from Master in last 5sec, electing new master");
            drop(last_lifesign_locked);
            become_master(me,state);
            return true;
        }
        drop(last_lifesign_locked);
    }    
}


/// Wait ID*150ms before checking if the master role is taken, if not assume master role and broadcast worldview
/// 
/// # Arguments:
/// 
/// * `me` - &mut Elevator - mutable refrence to this elevator.
/// * `state` - &mut SystemState - mutable refrence to the system state.
///  
/// # Returns:
///
/// Returns - None - .
///
pub fn become_master(me: &mut Elevator,state: &mut SystemState){

    sleep(Duration::from_millis(150*u64::from(me.ID)));
    if check_master_failure(me, state){
        let mut active_elevators_locked = state.active_elevators.lock().unwrap();
        if let Some(old_master) = active_elevators_locked.iter_mut().find(|e| e.ID == state.master_ID) {
            old_master.role = Role::Slave;
            println!("Old master (ID: {}) set to Slave.", old_master.ID);
        }
        if let Some(new_master) = active_elevators_locked.iter_mut().find(|e|e.ID == state.me_ID){
            new_master.role = Role::Master;
            println!("New master (ID: {}).", new_master.ID);

            let message = make_Udp_msg(MessageType::New_Master, &vec![new_master.clone()]);
            udp_broadcast(&message);
        } else {
            println!("ERROR: New master is not an active elevator")
        }
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
