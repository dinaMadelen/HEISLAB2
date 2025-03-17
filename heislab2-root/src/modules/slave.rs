//! ## Slave Module
//! This module provides structs and functions for slave nodes
//! 
//! ## The structs includes:
//! - **last_lifesign**: Time instance of when the last message from master was recived.
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

//-----------------------IMPORTS------------------------------------------------------------

use crate::modules::elevator_object::elevator_init::Elevator; //Import for elevator struct
use crate::modules::udp::{UdpMsg, MessageType, udp_send_ensure, udp_broadcast, make_Udp_msg, udp_ack};



use std::net::{UdpSocket, SocketAddr};
use std::thread::sleep;
use std::time::{Instant, Duration}; //https://doc.rust-lang.org/std/time/struct.Instant.html
use std::thread; // imported in elevator.rs, do i need it here?
use std::env; // Used for reboot function
use std::process::{Command, exit}; //Used for reboot function
use crate::modules::order_object::order_init::Order;



static mut failed_orders: Vec<Order> = Vec::new(); //MAKE THIS GLOBAL

//-----------------------STRUCTS------------------------------------------------------------

/// Tracks last lifesign from master 
struct Lifesign {
    last_lifesign: Instant,
}


//----------------------Fucntions-----------------------------------------------------------

/// recive_order
/// Recive order from master and add it to queue if it is not already in the queue
/// then ack the master that the order has been added
/// 
/// # Arguments:
/// 
/// * `slave` - &mut Elevator - &refrence to the elevator where the order should be addded.
/// * `new_order` - u8 - floor that should be added to the queue.
/// * `socket` - &UdpSocket - socket of the sender.
/// * `master_adress` - SocketAddr - adress where the master is expecting the ack.
/// * `original_msg` - &UdpMsg - refrence to the udpmessage where the slave recived the order.
/// 
/// # Returns:
///
/// Returns - bool- 'true' if order has been added to queue or the order already was in the queue and ackowledgement has been sent, if the acknowledgement failed it returns 'false' 
///
pub fn receive_order(slave: &mut Elevator, new_order: Order, socket: &UdpSocket, master_address: SocketAddr, original_msg: &UdpMsg) -> bool {
    
    if !slave.queue.contains(&new_order) {
        slave.queue.push(new_order.clone());
        println!("{} added to elevator {}", new_order.floor, slave.ID);
        return udp_ack(socket, master_address, original_msg, slave.ID);
    }else{
        println!("{} already in queue for elevator {}", new_order.floor, slave.ID);
        return udp_ack(socket, master_address, original_msg, slave.ID);
    }
}

/// notify_completed
/// Broadcast that an order is completed
/// 
/// # Arguments:
/// 
/// * `slave_id` - u8 - ID of the elevator that completed the order.
/// * `order` - u8 - floornumber of the completed order.
/// 
/// # Returns:
///
/// Returns - bool - 'true' if succsessful broadcast, 'false' if failed to broadcast.
///
pub fn notify_completed(slave_id: u8, order: Order) -> bool {

    let message = make_Udp_msg(slave_id, MessageType::Order_Complete, elevator:&Elevator);
    return udp_broadcast(&message);
}

/// cancel_order
/// Remove an active order from a queue
/// 
/// # Arguments:
/// 
/// * `slave` - &mut Elevator - refrence to the elevator where the order should be removed from.
/// * `order` - u8 - floor that should be removed from queue.
/// 
/// # Returns:
///
/// Returns - bool - returns 'true' if the order was successuly removed, returns 'false' if the floor couldnt be found in the queue.
///
pub fn cancel_order(slave: &mut Elevator, order: u8) -> bool {

    if let Some(index) = slave.queue.iter().position(|o| o.floor == order) {
        slave.queue.remove(index);
        println!("Order {} removed from queue of elevator {}", order, slave.ID);
        return true;
    }
    println!("Order {} couldnt be found in queue of elevator {}", order, slave.ID);
    return false;
}

/// update_from_worldview
/// Checks for discrepancies between the elevators worldview and the masters worldview
/// if there are orders in the worldview that do not exist in the queue , it updates the elevator's order queue based on a received worldview.
/// if there are missing orders in the worldview, it notifies the master that there are missing orders.
/// 
/// # Arguments:
/// 
/// * `active_elevators` - &mut Vec<Elevator> - refrence to list of active elevatosrs.
/// * `new_worldview` - &worlcview) - worldview struct.
/// 
/// 
/// # Returns:
///
/// Returns -bool - returns 'true' if added orders or orders match, returns 'false' if there are missing orders in worldview.
///
pub fn update_from_worldview(active_elevators: &mut Vec<Elevator>, new_worldview: &Worldview) -> bool {

    let mut worldview_changed = false;

    for wv_elevator in &new_worldview.elevators{
        if let Some(elevator) = active_elevators.iter_mut().find(|e| e.ID == wv_elevator.ID){

            let active_queue=elevator.queue.clone();

            //No new orders
            if active_queue == new_elevaotor.queue{
                println!("Worldview matches for ID:{}", elevator.ID);
                continue;
            }

            //Found missing order, add them to queue
            let missing_orders: Vec<Order> = wv_elevator.queue.iter().filter(|&order| !active_queue.contains(order)) .cloned().collect();
            if !missing_orders.is_empty() {
                println!("Elevator {} is missing orders {:?}. Adding...", active_elevator.ID, missing_orders);
                elevator.queue.extend(missing_orders);
                worldview_changed = true;
            }

        } else{
            // Add missing worldview elevator to active elevators
            println!("Found missing elevator, Adding new elevator ID {} from worldview.", new_elevator.ID);
            active_elevators.push(new_elevator.clone());
            worldview_changeed = true;
        }
    }   
    return worldview_changed;
}

/// Missing order in worldview, notify master that there is a missing order/orders
/// 
/// # Arguments:
/// 
/// * `` -  - .
/// 
/// # Returns:
///
/// Returns - - .
///
pub fn notify_worldview_error(slave_id: u8, missing_orders: Vec<Elevator>) {

    let message = make_Udp_msg(slave_id, MessageType::Error_Worldview, missing_orders);
    udp_send(&socket, &master_address.to_string(), &message);
}


/// Check for worldview, no update in given time 5s?, assumes dead master and starts master election
/// 
/// # Arguments:
/// 
/// * `` -  - .
/// 
/// # Returns:
///
/// Returns - - .
///
pub fn check_master_failure() -> bool {

    loop{
        sleep(Duration::from_millis(5000));
        if  last_lifesign_master>Duration::from_millis(5000) {
            println!("Master not broadcasting, electing new master");
            set_new_master(&mut me);
            return true;
        }
    }    
    println!("Master still alive");
    return false;
}


/// Wait ID*150ms before checking if the master role is taken, if not assume master role and broadcast worldview
/// 
/// # Arguments:
/// 
/// * `` -  - .
/// 
/// # Returns:
///
/// Returns - - .
///
pub fn set_new_master(me: &mut &Elevator){

    sleep(Duration::from_millis((150*&me.ID.into())));
        if check_master_failure(){
            if let Some(old_master) = active_elevators.iter_mut().find(|e| matches!(e.role, Role::Master)) {
                old_master.role = Role::Slave;
                println!("Old master (ID: {}) set to Slave.", old_master.ID);
            }
            me.role = Role::Master;
            println!("New master (ID: {}).", me.ID);

            let message = make_Udp_msg(me, MessageType::New_Master, vec![]);
            udp_broadcast(&message);
        }
    
}

/// Starts a new instance and kills the old instance of the program
/// 
/// # Arguments:
/// 
/// * `` -  - .
/// 
/// # Returns:
///
/// Returns - - .
///
pub fn reboot_program(){

    Command::new(env::current_exe().expect("Failed to find path to program"))
        .spawn()
        .expect("Failed to restart program, Restart program manually");
    exit(0); // Kill myself
}
