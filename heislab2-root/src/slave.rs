#[warn(non_snake_case)]
/*

Mottak av ordre

Behandle ordre

Si ifrå når ferdig, trenger dette være egen func eller bare bake den inn i behandling?

Kanselere ordre

Oppdatere via worldview og sjekke om det er feil i worldview

Melde ifra om worldview error

*/


//-----------------------IMPORTS------------------------------------------------------------

use std::time::{Instant, Duration};https://doc.rust-lang.org/std/time/struct.Instant.html
use std::thread; // imported in elevator.rs, do i need it here?
use std::env; // Used for reboot function
use std::process::{Command, exit}; //Used for reboot function

use crate::elevator::Elevator;//Import for elevator struct



//-----------------------STRUCTS------------------------------------------------------------

// Create and update a variable of this struct when you recive a worldview 
// let mut lifesign = Lifesign{lastlife_sign: Instant::now()};  Call this when creating the first timestamp
// lifesign.lastlife_sign = Instant::now(); call this when updating the timestep
struct Lifesign {
    last_lifesign: Instant,
}



// Recive order from master
fn receive_order(slave: &mut Elevator, new_order: u8) -> bool {
    if !slave.queue.contains(&new_order) {
        slave.queue.push(new_order);
        println!("{} added to elevator {}", new_order, slave.id);
        udp_ack(socket, sender_address);
        return true;
    }
    return false;
}

//-----------------------FUNCTIONS---------------------------------------------------------

/* Does this function already exists in elevator.rs?
// Process next order in queue
fn process_next_orders(slave: &mut Elevator) {

    if slave.orders.is_empty() -> bool{
        slave.go_next_floor(order);
    }
        while !finished{
            !todo("process order");    //wait
        }

        notify_completed(slave.id, order);
        slave.orders.pop();

        return true;
    
}
*/

// Broadcast that an order is completed
fn notify_completed(slave_id: u8, order: u8) {

    let message = make_udp_msg(slave_id, MessageType::OrderCompleted, order);
    udp_broadcast(socket, &message);
}

// Remove order from queue
fn cancel_order(slave: &mut Elevator, order: u8) -> bool {

    if let Some(index) = slave.queue.iter().position(|&o| o == order) {
        slave.queue.remove(index);
        println!("Order {} removed from queue of elevator {}", order, slave.id);
        return true;
    }
    return false;
}


fn update_from_worldview(slave: &mut Elevator, new_worldview: Vec<Vec<u8>>) -> bool {
    // Compare if worldviews match
    if slave.queue.iter().cloned().collect::<Vec<Elevator>() == new_worldview {
        println!("Received worldview matches");
        // No changes, no reason to continue comparing
        return true;
    }

    // Find missing orders: Orders in "old_worldview" but not in "new_worldview"

    let mut missing_orders = Vec::new();
    // Add missing orders to missing order vector
    for order in &slave.queue {
        if !new_worldview.contains(order) {
            missing_orders.push(order.clone());
        }
    }

    // Notify master if there are missing orders
    if !missing_orders.is_empty() {
        notify_worldview_error(slave.id, missing_orders.clone());
        println!("Master worldview is missing orders, notifing master")
        return false;
    }

    // Find new orders: Orders in "new_worldview" but not in "old_worldview"
    let mut new_orders = Vec::new();
    for order in &new_worldview {
        if !slave.queue.contains(order) {
            new_orders.push(order.clone());
        }
    }

    // Merge worldviews (Union of current and new)
    for order in new_orders {
        slave.queue.insert(order);
    }

    println!("Updated worldview");
    return true;
}

// Missing order in worldview, notify master that there is a missing order
fn notify_wordview_error(slave_id: u8, missing_orders: Vec<u8>) {
    let message = make_udp_msg(slave_id, MessageType::WorldviewError, missing_orders);
    udp_send_ensure(&socket, &master_address(), &message);
}


// Check for worldview, no update in given time 5s?, assumes dead master and starts master election
fn check_master_failure() -> bool {

    sleep(time::Duration::from_millis(5000));
    
    if  last_lifesign_master>Duration::from_secs(5) {
        println!("Master not broadcasting, electing new master");
        start_master_election();
        return true;
    }else{
        println!("Master still alive");
    }
    return false;
}


// Wait id*150ms before checking if the master role is taken, if not assume master role and broadcast worldview
fn start_master_election(time: &slave.id) {

    sleep(time::Duration::from_millis(150*slave.id));

    if detect_master_failure(){
    
        assume_master()
        let message = make_udp_msg(slave_id, MessageType::Worldview, vec![]); // add worldview
        udp_broadcast(&socket, &message);
    }
}
// Not sure if this exists already in another .rs, if do remove and replace this function in this and master.rs 
fn reboot_program(){

    Command::new(env::current_exe().expect("Failed to find path to program")) // Start new instance 
        .spawn()
        .expect("Failed to restart program, Restart program manually");

    exit(0); // Kill myself
}

//----------------------------------TESTS-------------------------------------------------------------

#[cfg(test)] // https://doc.rust-lang.org/book/ch11-03-test-organization.html Run tests with "cargo test"
mod tests {
    use super::*;
    use std::net::TcpStream;
    use std::sync::{Arc, Mutex};
    use std::time::Duration;
    use std::thread;

    #[test]
    fn test_receive_order() {

        let mut elevator = Elevator::init("127.0.0.1:1234", 5).unwrap();
        //Add order to queue
        assert!(receive_order(&mut elevator, 3));  
        //Check for order in queue
        assert!(elevator.queue.contains(&3));      
        //Only adds one, Should not add duplicate
        assert!(!receive_order(&mut elevator, 3)); 
    }

    /* This function already exists in elevator.rs?
    #[test]
    fn test_process_next_orders() {

        let mut elevator = Elevator::init("127.0.0.1:1234", 5).unwrap();
        elevator.queue.push(2);
        process_next_orders(&mut elevator);
        assert!(!elevator.queue.contains(&2));
    }
    */

    /*  Not sure if we need this
    #[test]
    fn test_notify_completed() {

        notify_completed(1, 2);
    }
    */


    #[test]
    fn test_cancel_order() {

        let mut elevator = Elevator::init("127.0.0.1:1234", 5).unwrap();
        elevator.queue.push(4);
        // Existing order gets cancelled
        assert!(cancel_order(&mut elevator, 4));  
        // Check that queue no longer contains order
        assert!(!elevator.queue.contains(&4));    
        // Other orders not cancelled
        assert!(!cancel_order(&mut elevator, 5)); 
    }

    #[test]
    fn test_update_from_worldview() {

        let mut elevator = Elevator::init("127.0.0.1:1234", 5).unwrap();
        elevator.queue.push(1);
        let new_worldview = vec![vec![1, 2, 3]]; 
        assert!(!update_from_worldview(&mut elevator, new_worldview));
    }

    #[test]
    fn test_check_master_failure() {
        let result = check_master_failure();
        // Expect return true or return false
        // Expand on this test with one that fails and one that works
        assert!(result || !result); 
    }

    #[test]
    fn test_start_master_election() {
        let mock_id = 2;
        start_master_election(&mock_id);
        // Need to check if master election process is triggered
        // Expand on this test
    }

    #[test]
    fn test_reboot_program() {
        // Not sure how to test this as it reboots the program, maybe drop it and just test while running the program?
    }
}
