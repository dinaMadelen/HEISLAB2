#[warn(non_snake_case)]

use std::time::{Instant, Duration};https://doc.rust-lang.org/std/time/struct.Instant.html
use std::thread; // imported in elevator.rs, do i need it here?


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
        udp_ack(socket: &UdpSocket, target_address: SocketAddr); // fill inn correct socket and adress
        return true;
    }
    return false;
}

// Process next order in queue
fn process_orders(slave: &mut Elevator) {
    while let Some(order) = slave.queue.pop() {
        slave.go_next_floor(order);
        notify_completed(slave.id, order);
    }
}

// Broadcast that an order is completed
fn notify_completed(slave_id: u8, order: u8) {

    let message = make_udp_msg(slave_id, MessageType::OrderCompleted, order);
    udp_broadcast(&socket, &message);
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
    if slave.queue.iter().cloned().collect::<Vec<Vec<u8>>>() == new_worldview {
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
        notify_master_error(slave.id, missing_orders.clone());
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
    udp_send(&socket, &master_address(), &message);
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
        let message = make_udp_msg(slave_id, MessageType::Worldview, vec![]);
        udp_broadcast(&socket, &message);
    }
}
