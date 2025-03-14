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

use crate::modules::elevator::Elevator; //Import for elevator struct
use crate::modules::udp::{UdpMsg, MessageType, udp_send_ensure, udp_broadcast, make_Udp_msg, udp_ack};
use std::net::{UdpSocket, SocketAddr};
use std::thread::sleep;
use std::time::{Instant, Duration}; //https://doc.rust-lang.org/std/time/struct.Instant.html
use std::thread; // imported in elevator.rs, do i need it here?
use std::env; // Used for reboot function
use std::process::{Command, exit}; //Used for reboot function

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
fn receive_order(slave: &mut Elevator, new_order: u8, socket: &UdpSocket, master_address: SocketAddr, original_msg: &UdpMsg) -> bool {
    
    if !slave.queue.contains(&new_order) {
        slave.queue.push(new_order);
        println!("{} added to elevator {}", new_order, slave.id);
        return udp_ack(socket, master_address, original_msg, slave.id);
    }else{
        println!("{} already in queue for elevator {}", new_order, slave.id);
        return udp_ack(socket, master_address, original_msg, slave.id);
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
fn notify_completed(slave_id: u8, order: u8) {

    let message = make_udp_msg(slave_id, MessageType::OrderCompleted, order);
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
fn cancel_order(slave: &mut Elevator, order: u8) -> bool {

    if let Some(index) = slave.queue.iter().position(|&o| o == order) {
        slave.queue.remove(index);
        println!("Order {} removed from queue of elevator {}", order, slave.id);
        return true;
    }
    println!("Order {} couldnt be found in queue of elevator {}", order, slave.id)
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
/// * `new_worldview` - Vec<Vec<u8>>) - Vectors of vectors containing the orders, the outer vector holds each queue. .
/// 
/// 
/// # Returns:
///
/// Returns -bool - returns 'true' if added orders or orders match, returns 'false' if there are missing orders in worldview.
///
fn update_from_worldview(active_elevators: &mut Vec<Elevator>, new_worldview: Vec<Vec<u8>>) -> bool {

    for elevator in active_elevators.iter_mut(){
        

        let mut elevator_queue_snapshot = elevator.queue.clone();
        
        //Sort orders
        let mut all_orders: Vec<u8> = new_worldview.iter().flatten().cloned().collect();
        all_orders.sort();
        all_orders.dedup();

        if elavtor_queue_snapshot == all_orders {
            // No need to change worldview for this order, check next elevator
            println!("Received worldview matches for ID {}" elvator.ID);
            continue;
        }

        let missing_orders: Vec<u8> = elevator_queue_snapshot.iter()
            // Find orders that are missing from queues.
            .filter(|&&order| !all_orders.contains(&order))
            .cloned()
            .collect();

        if !missing_orders.is_empty() {
            // No missing orders in queues, must be missing from worldview, notify master
            notify_worldview_error(elevator.id, &missing_orders);
            println!("Master worldview is missing orders, notifying master");
            return false;
        }

        let new_orders: Vec<u8> = all_orders.iter()
            .filter(|&&order| !elevator_queue_snapshot.contains(&order))
            .cloned()
            .collect();

        elevator.queue.extend(new_orders);
        println!("Updated worldview");
    }
        // Merge worldviews (Union of current and new)
        for order in new_orders {
            elevator.queue.insert(order);
        }
        println!("Updated worldview");
    return true;

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
fn notify_worldview_error(slave_id: u8, missing_orders: Vec<u8>) {

    let message = make_udp_msg(slave_id, MessageType::WorldviewError, missing_orders);
    udp_send_ensure(&socket, &master_address.to_string(), &message);
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
fn check_master_failure() -> bool {

    sleep(time::Duration::from_millis(5000));
    
    if  last_lifesign_master>Duration::from_secs(5) {
        println!("Master not broadcasting, electing new master");
        set_new_master();
        return true;
    }
    println!("Master still alive");
    return false;
}


/// Wait id*150ms before checking if the master role is taken, if not assume master role and broadcast worldview
/// 
/// # Arguments:
/// 
/// * `` -  - .
/// 
/// # Returns:
///
/// Returns - - .
///
set_new_master(&mut me);{

    loop{
    sleep(Duration::from_millis(150*&me.id));
        if detect_master_failure(){
            //old_master.role = Role::Slave //i have to fix this as there is no role in elevator struct
            //me.role = Role::Master// same here
            let message = make_udp_msg(me.id, MessageType::Worldview, vec![]);
            udp_broadcast(&socket, &message);
            break;
        }
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
fn reboot_program(){

    Command::new(env::current_exe().expect("Failed to find path to program"))
        .spawn()
        .expect("Failed to restart program, Restart program manually");
    exit(0); // Kill myself
}
