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


/// Recive order from master and add it to queue if it is not already in the queue
/// 
/// # Arguments:
/// 
/// * `` -  - .
/// 
/// # Returns:
///
/// Returns - - .
///
fn receive_order(slave: &mut Elevator, new_order: u8, socket: &UdpSocket, sender_address: SocketAddr, original_msg: &UdpMsg) -> bool {
    
    if !slave.queue.contains(&new_order) {
        slave.queue.push(new_order);
        println!("{} added to elevator {}", new_order, slave.id);
        udp_ack(socket, sender_address, original_msg, slave.id);
        return true;
    }
    return false;
}


/// Broadcast that an order is completed
/// 
/// # Arguments:
/// 
/// * `` -  - .
/// 
/// # Returns:
///
/// Returns - - .
///
fn notify_completed(slave_id: u8, order: u8) {

    let message = make_udp_msg(slave_id, MessageType::OrderCompleted, order);
    udp_broadcast(&message);
}

/// Remove an active order from a queue
/// 
/// # Arguments:
/// 
/// * `` -  - .
/// 
/// # Returns:
///
/// Returns - - .
///
fn cancel_order(slave: &mut Elevator, order: u8) -> bool {

    if let Some(index) = slave.queue.iter().position(|&o| o == order) {
        slave.queue.remove(index);
        println!("Order {} removed from queue of elevator {}", order, slave.id);
        return true;
    }
    return false;
}

/// Updates the elevator's order queue based on a received worldview.
/// 
/// # Arguments:
/// 
/// * `` -  - .
/// 
/// # Returns:
///
/// Returns - - .
///
fn update_from_worldview(slave: &mut Elevator, new_worldview: Vec<Vec<u8>>) -> bool {
    let slave_queue_snapshot = slave.queue.clone();
    
    let mut all_orders: Vec<u8> = new_worldview.iter().flatten().cloned().collect();
    all_orders.sort();
    all_orders.dedup();

    if slave_queue_snapshot == all_orders {
        println!("Received worldview matches");
        return true;
    }

    let missing_orders: Vec<u8> = slave_queue_snapshot.iter()
        .filter(|&&order| !all_orders.contains(&order))
        .cloned()
        .collect();

    if !missing_orders.is_empty() {
        notify_worldview_error(slave.id, &missing_orders);
        println!("Master worldview is missing orders, notifying master");
        return false;
    }

    let new_orders: Vec<u8> = all_orders.iter()
        .filter(|&&order| !slave_queue_snapshot.contains(&order))
        .cloned()
        .collect();

    slave.queue.extend(new_orders);
    println!("Updated worldview");
    true
}

    // Merge worldviews (Union of current and new)
    for order in new_orders {
        slave.queue.insert(order);
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

//----------------------------------TESTS-------------------------------------------------------------

#[cfg(test)] // https://doc.rust-lang.org/book/ch11-03-test-organization.html Run tests with "cargo test"
mod tests {
    use super::*;
    use std::net::{UdpSocket, SocketAddr};
    use std::sync::{Arc, Mutex};
    use std::thread;
    use std::time::{Duration, Instant};

    /// Helper function to create a test UdpSocket
    fn create_test_socket() -> UdpSocket {
        UdpSocket::bind("127.0.0.1:0").expect("Failed to bind test UDP socket")
    }

    /// Helper function to create a test Elevator
    fn create_test_elevator() -> Elevator {
        Elevator::new(1) // Assuming `Elevator::new(id)` initializes an elevator with ID
    }

    #[test]
    fn test_receive_order() {
        let mut elevator = create_test_elevator();
        let socket = create_test_socket();
        let sender_address = "127.0.0.1:4000".parse().unwrap();
        let original_msg = UdpMsg::new(1, MessageType::New_Order, vec![3]);

        assert!(receive_order(&mut elevator, 3, &socket, sender_address, &original_msg));
        assert!(elevator.queue.contains(&3));

        // Should not add a duplicate order
        assert!(!receive_order(&mut elevator, 3, &socket, sender_address, &original_msg));
    }

    #[test]
    fn test_notify_completed() {
        let socket = create_test_socket();
        let slave_id = 1;
        let order = 2;

        // Just test that it does not panic
        notify_completed(slave_id, order);
    }

    #[test]
    fn test_cancel_order() {
        let mut elevator = create_test_elevator();
        elevator.queue.push(4);
        assert!(cancel_order(&mut elevator, 4));
        assert!(!elevator.queue.contains(&4));

        // Cancel a non-existent order
        assert!(!cancel_order(&mut elevator, 5));
    }

    #[test]
    fn test_update_from_worldview() {
        let mut elevator = create_test_elevator();
        elevator.queue.push(1);
        let new_worldview = vec![vec![1, 2, 3]];

        assert!(!update_from_worldview(&mut elevator, new_worldview.clone()));
        elevator.queue = vec![1, 2, 3];
        assert!(update_from_worldview(&mut elevator, new_worldview));
    }

    #[test]
    fn test_notify_worldview_error() {
        let socket = create_test_socket();
        let slave_id = 1;
        let missing_orders = vec![3, 4];
        notify_worldview_error(slave_id, &missing_orders);
    }

    #[test]
    fn test_check_master_failure() {
        let mut elevator = create_test_elevator();
        let last_lifesign_master = Instant::now();

        // Should return false since master is alive
        assert!(!check_master_failure(last_lifesign_master, &mut elevator));

        // Simulate expired master heartbeat
        let expired_time = Instant::now() - Duration::from_secs(6);
        assert!(check_master_failure(expired_time, &mut elevator));
    }

    #[test]
    fn test_set_new_master() {
        let mut elevator = create_test_elevator();

        // Before assuming master role
        assert!(matches!(elevator.role, Role::Slave));

        // Set new master
        set_new_master(&mut elevator);

        // After assuming master role
        assert!(matches!(elevator.role, Role::Master));
    }

    #[test]
    fn test_reboot_program() {
        // Not sure how to test this as it reboots the program, maybe drop it and just test while running the program?
        !todo("Figure out a way to test reboot_program");
    }

}


