
use std::time::{Instant, Duration};https://doc.rust-lang.org/std/time/struct.Instant.html
use std::thread; // imported in elevator.rs, do i need it here?

struct MasterStatus {
    last_lifesign_master: Instant,
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
        move_elevator_to(order);
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

// Compare hash of new worldview with old.
fn update_from_worldview(slave: &mut Elevator, new_worldview: Vec<Vec<u8>>) {


    if current_worldview_hash==new_worldview_hash{
        // No changes in worldview
        print("Recived worldview matches")
        return true;
    }

    let current_worldview: HashSet<u8> = slave.queue.iter().cloned().collect();
    let new_worldview_set: HashSet<u8> = new_worldview.iter().cloned().collect();

    // Identify missing orders (orders in current worldview but NOT in new worldview)
    let missing_orders: Vec<u8> = current_worldview.difference(&new_worldview_set).cloned().collect();
    
    // Identify new orders (orders in new worldview but NOT in current worldview)
    let new_orders: Vec<u8> = new_worldview_set.difference(&current_worldview).cloned().collect();
    
    // Notify master about missing orders
    if !missing_orders.is_empty() {
        notify_master_discrepancy(slave.id, missing_orders.clone());
    }

    // Merge worldviews (Union)
    slave.queue = current_worldview.union(&new_worldview_set).cloned().collect();
    
    println!(
        "Updated worldview for elevator {}. New queue: {:?}",
        slave.id, slave.queue
    );
}


// Missing order in worldview, notify master that there is a missing order
fn notify_wordview_error(slave_id: u8, missing_orders: Vec<u8>) {
    let message = make_udp_msg(slave_id, MessageType::WorldviewDiscrepancy, missing_orders);
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
