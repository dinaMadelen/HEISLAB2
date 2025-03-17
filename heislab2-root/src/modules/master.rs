//! ## Master Module
//! This module provides structs and functions for the master node
//! 
//! ## The structs includes:
//! - **Worldview**
//! - **Role**
//! 
//! ## The functions includes:
//! - 'give_order'
//! - 'remove_from_queue'
//! - 'correct_master_worldview'
//! - 'master_worldview
//! - 'handle_slave_failure'
//! - 'reassign_orders'
//! - 'best_to_worst_elevator'
//! - 'handle_multiple_masters'
//! 
//! ## Dependencies
//! 
//! ```toml
//! [dependencies]
//! ```

//the comments are verbose so we can autogenerate documentation using 'cargo doc' https://blog.guillaume-gomez.fr/articles/2020-03-12+Guide+on+how+to+write+documentation+for+a+Rust+crate

#[warn(non_snake_case)]

//-----------------------IMPORTS------------------------------------------------------------
use crate::modules::udp::{UdpHandler,udp_send_ensure,udp_broadcast,udp_receive,make_Udp_msg};
use crate::modules::elevator_object::elevator_init::{Elevator,Status};
use crate::modules::slave::{reboot_program};
use crate::modules::order_object::order_init::Order;


use serde::{Serialize, Deserialize};
use std::net::UdpSocket;


static mut failed_orders: Vec<Orders> = Vec::new(); //MAKE THIS GLOBAL

/* 
//-----------------------GLOBAL VARIABLES---------------------------------------------------
static mut FAILED_ORDERS: Option<Arc<Mutex<Vec<Order>>>> = None;
*/


//-----------------------STRUCTS------------------------------------------------------------

/// A struct that holds all the active elevators aswell as all active lights
pub struct Worldview{

    worldview_elevators: Vec<Elevator>,
    lights: Vec<u8>,
}

/// All possible roles of a node
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum Role{
    Master,
    Slave,
    Error,
}

//-----------------------FUNCTIONS---------------------------------------------------------

/// give_order
/// Sends an order to a slave elevator and waits for an acknowledgment.
/// Broadcast order and wait for responce from reciver, if not recived resend, if this fail. find return false
/// The diffrence from just adding from worldview broadcast and from give_order() is that unlike regular udp_broadcast() give_order() requires an acknoledgement from the recivers
/// 
/// # Arguments:
/// 
/// * `master` - &Elevator - A reference to the master `Elevator` initiating the order.
/// * `elevator` - u8 - The ID of the elevator receiving the order.
/// * `new_order` - Vec<Order> - floor number of the new order.
/// * `socket` - &UdpSocket - refrence to sender socket.
/// * `active_elevator` - &Vec<Elevator> - refrence to list of active elevators.
/// 
/// # Returns:
///
/// Returns - bool- `true` if the order was successfully acknowledged, otherwise `false`.
///
pub fn give_order(master: &Elevator, elevator:&Elevator, new_order: Vec<&Order>, active_elevators: &Vec<Elevator>) -> bool {

    let mut retries = 3;
    let max_timeout_ms = 300;
    let mut received_acks = Vec::new();

    let message = make_Udp_msg(master.ID, MessageType::NewOrder, new_order);

    let mut missing_acks: Vec<u8> = active_elevators.iter().map(|e| e.ID).collect(); // https://doc.rust-lang.org/beta/std/iter/trait.Iterator.html


    println!("Broadcasting new order floor:{:?} elevator:{}",order, elevator.ID);

    udp_broadcast(&message);

    missing_acks.retain(|id| !received_acks.contains(id));

    if !missing_acks.is_empty(){
        while retries > 0 {
            
            println!("Remaining retries {}: order {:?} to elevators {:?}", retries, new_order, missing_acks);
            retries -= 1;
            let mut received_acks = Vec::new();

            let start_time = std::time::Instant::now();

            while start_time.elapsed() < Duration::from_millis(max_timeout_ms){
                if let Some(response) = UdpHandler.recive(max_timeout_ms, slave, me, worldview) {
                    if response.header.message_type == MessageType::Ack && response.header.checksum == message.header.checksum{
                            received_acks.push(response.header.sender_id);
                            
                        }
                }


                // If all expected ACKs are received, return early
                if missing_acks.is_empty(){
                    println!("All elevators acknowledged order {:?}.", new_order);
                    return true;
                }                
            }

            
            // Remove recived acks from missing list
            missing_acks.retain(|id| !received_acks.contains(id));


            // Send the order to all elevators where ack has not been recvied
            for &elevator_id in &missing_acks {
            let target_address = &elevator.ID;
                UdpHandler.send(socket, &target_address, &message);
            }
            
        }

        if missing_acks.is_empty() {
            println!("Order {:?} successfully acknowledged by all elevators.", new_order);
            return true;
        } else {
            println!("Missing acknowledgments from: {:?}. Retrying...", missing_acks);
        }
    }

    println!("Failed to deliver order {:?} after {} retries to {:?}.", new_order, retries, missing_acks[0]);
    return false;
}

///remove_from_queue
/// Broadcast order to remove one or more orders from a specific elevator
///
/// # Arguments:
/// 
/// * `slave_id` - u8 - ID of the elevator where the order/orders should be removed from.
/// * `removed_orders` - Vec<u8> - Vector of orders that will be removed.
/// 
/// # Returns:
///
/// Returns - bool- `true` if the order was successfully acknowledged, otherwise `false`.
///
pub fn remove_from_queue(slave: &mut Elevator, removed_orders: Vec<Elevator>) -> bool {

    let message = make_Udp_msg(me.ID, MessageType::RemoveOrder, removed_orders);
    return udp_send_ensure(&socket, &slave.inn_address, &message, 3, sent_messages);
}


/// correct_master_worldview
/// Compare message and send out the corrected worldview (union of the recived and current worldview)
/// 
/// # Arguments:
/// 
/// * `master` - &Elevator - Refrence to master elevator.
/// * `missing_orders` - Vec<Elevator> - List of elevators with errors in queues.
/// * `active_elevators` - &mut Vec<Elevator> - Refrence to master elevator.
/// 
/// # Returns:
///
/// Returns - bool- `true` if the order was successfully acknowledged, otherwise `false`.
///
pub fn correct_master_worldview(master: &mut Elevator, missing_orders:&mut Vec<Elevator>, active_elevators: &mut Vec<Elevator>) -> bool {
    println!("Correcting worldview for master {}", master.ID);

    let mut changes_made = false;

    if missing_orders.is_empty(){
        println!("List of missing orders is empty");
        return false;
    }

    // Compare active elevators to missing orders list
    for missing_elevator in missing_orders.iter_mut() {
        if let Some(elevator) = active_elevators.iter_mut().find(|e| e.ID == missing_elevator.ID) {
            for order in &missing_elevator.queue {
                if !elevator.queue.contains(&order) {
                    elevator.queue.push(order.clone());
                    println!("Added missing order {:?} to elevator {}", order.floor, elevator.ID);
                    changes_made = true;
                }
            }
        } else {
            println!(
                "Warning: Elevator ID {} from missing_orders not found in active elevators",
                missing_elevator.ID
            );
        }
    }
    return changes_made;
}



/// generate_worldview
/// Create worldview from list of active elevators
/// finds active lights from orders of active elevators
/// 
/// # Arguments:
/// 
/// * `elevators` - &Vec<Elevator> - Refrence to list of active elevators
/// 
/// # Returns:
///
/// Returns - Worldview- Returns a worldview struct.
///
pub fn generate_worldview(active_elevators: &Vec<Elevator>) -> Worldview {

    // Find active lights
    let mut lights = Vec::new();
    for elevator in active_elevators {

        for &order in &elevator.queue {
            let floor = order.floor;
            if !lights.contains(&floor) {
                lights.push(floor);
            }
        }
    }
    
    lights.sort();
    // No duplicates
    lights.dedup();

    return Worldview {
        elevators: elevators.clone(), 
        lights,                       
    };
}


/// master_worldview
/// Compare message and send out the corrected worldview (union of the recived and current worldview)
/// 
/// # Arguments:
/// 
/// * `master` - &Elevator - Refrence to master elevator.
/// 
/// # Returns:
///
/// Returns - bool- `true` if the order was successfully broadcasted, otherwise `false`.
///
pub fn master_worldview(master:Elevator) -> bool{

    let current_worldview = generate_worldview(&elevators);
    
    let message = make_Udp_msg(master.ID,MessageType::Wordview, data); 
    
    return udp_broadcast(&message);
}

// Give away master role, NOT NEEDED, KILL INSTEAD
/*
fn relinquish_master(master: &mut Elevator) -> bool {

    let message = make_udp_msg(master.ID, MessageType::RelinquishMaster, vec![]);
    udp_broadcast(&socket, &message);

    master.status = Elevator.status::Error;
    return true;
}
*/


/// handle_slave_failure
/// Handle slave failure, take action to secure service for orders when a slave goes offline
/// 
/// # Arguments:
/// 
/// * `slave_id` - u8 - ID of the elevator that has failed.
/// * `elevators` - &mut Vec<Elevator> - refrence to the vector containing the active elevators.
/// 
/// # Returns: 
/// 
/// Returns - bool - `true` if orders have been succsessfully distributed and elevator har been removed from active elevators
/// else returns `false`
///
///
pub fn handle_slave_failure(slave_id: u8, elevators: &mut Vec<Elevator>)  -> bool {

    println!("Elevator {} is offline, redistributing elevator {}'s orders.", slave_id,slave_id);

    // Find and redistribute orders for elevator with that spesific ID
    if let Some(index) = elevators.iter().position(|elevator| elevator.ID == slave_id) {
        // Have to use clone to not take ownership of the queue variable(problem compiling)
        let orders = elevators[index].queue.clone(); 
        elevators.remove(index);
        reassign_orders(orders_u8, master, socket, elevators, &mut failed_orders);
        return true;
    } else {
        println!("Error: cant find Elevator with ID {}", slave_id);
        return false;
    }
}

/// Reassign order
/// Reassigns a or more orders from one elevator to active elevators
///  
/// # Arguments:
/// 
/// * `orders` - Vec<u8> - orders to be distributed.
/// * `master` - refrence to master
/// * `active_elevators` - &mut Vec<Elevator> - refrence to the vector containing the active elevators.
/// * `socket` - &UdpSocket- socket
/// * `failed_orders` - list of orders that failed to distribute.
/// 
/// # Returns:
/// 
/// returns nothing.
///
pub fn reassign_orders(orders: Vec<Order>,master: &Elevator,active_elevators: &Vec<Elevator>,failed_orders: &mut Vec<Order> ) -> bool {
    for order in orders {
        let mut assigned = false;

        for best_alternative in best_to_worst_elevator(&order, active_elevators) {
            println!("Assigning order {} to elevator {}", order.floor, best_alternative);

            if give_order(master, &active_elevators[best_alternative as usize], vec![&order], active_elevators) {
                println!("Order {} successfully reassigned to elevator {}", order.floor, best_alternative);
                assigned = true;
                break; // Stop trying if assigned successfully
            } else {
                println!("Failed to assign order {} to elevator {}. Trying next option", order.floor, best_alternative);
            }
        }

        // If no elevator accepted the order, store it for future retry
        if !assigned {
            println!("No available elevator for order {}. Storing for retry.", order.floor);
            failed_orders.push(order);
        }
    }
    if failed_orders.is_empty(){
        println!("All failed orders are redistributed");
        return true;
    }else{
        println!("There are orders that failed to be distributed");
        return false;
    }
}



/// Cost function that returns order to the best fitting elevators from best to worst alternative.
///  
/// # Arguments:
/// 
/// * `order` - u8 - the floor number.
/// * `elevators` - &Vec<Elevator> - refrence to list of active elevators that the functions will sort.
/// 
/// # Returns:
///
/// Retruns - Vec<u8> - a list of i IDs in decending order from best fit to worst fit.
///
pub fn best_to_worst_elevator(order: &Order, elevators: &Vec<Elevator>) -> Vec<u8> {

    // Vec<Elevator.ID, Score> Higher score = better alternative
    let mut scores: Vec<(u8, i32)> = Vec::new(); 


    // Give score to all active elevators
    for elevator in elevators {
        let mut score = 0;

        // Distance to the order (lower is better)
        score -= 10*(elevator.current_floor as i32 - order.floor as i32).abs();

        // Direction compatibility
        if elevator.status == Status::Moving {
            if (elevator.direction == 1 && elevator.current_floor < order.floor) || 
               (elevator.direction == -1 && elevator.current_floor > order.floor) {
                // Reward for moving towards the floor
                score += 10; 
            } else {
                // Penalty if moving away from the floor
                score -= 10; 
            }
        // Idle elevators are prefered over busy elevators
        }else if elevator.status == Status::Idle { 
            score += 20;
        }else if elevator.status == Status::Error {
            score -= 10000
        }

        // Shorter queue gets priority, Less is better
        score -= elevator.queue.len() as i32 * 5; 

        scores.push((elevator.ID, score));
    }

    // Sort by score
    scores.sort_by(|a, b| b.1.cmp(&a.1));

    // Return Vec<u8> of IDs in decending order from best to worst option  https://doc.rust-lang.org/std/iter/struct.Map.html
    return scores.into_iter().map(|(id, score)| id).collect();
}

/// handle_multiple_masters
/// If for some reason more than master is active, forexample race during election or one didnt recive the first message from new master.
/// master with lowest ID keeps the role, the rest become slaves.
/// 
/// # Arguments:
/// 
/// * `me` - &Elevator - refrence to this elevator.
/// * `sender` - &Elevator - refrence to senders elevator .
/// * `worldview` - &Worldview - .
/// 
/// 
/// # Returns:
///
/// Returns - Some(bool) - returns false if the ID is its own, returns true if it keeps the master if the ID is higher than the sender, reboots if it is lower.
///
pub fn handle_multiple_masters(me: &Elevator, sender: &Elevator) -> bool {

    let mut result = true;
    
        // This is the master, no others found
    if me.role == Role::Master {
        result = false; 

        // Give away master role, simple solution, Kill program and reboot
    }else if sender.ID < me.ID{
        reboot_program();
        result = true; // this never runs due to reboot, just here to stop warning
    } 
    return result; 

}
