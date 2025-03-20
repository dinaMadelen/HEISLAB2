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

use crate::modules::udp::udp::{udp_send_ensure,udp_broadcast,udp_receive,make_Udp_msg};
use crate::modules::elevator_object::elevator_init::{Elevator,Status};
use cargo::modules::slave::slave::{reboot_program};
use std::net::UdpSocket;


//-----------------------STRUCTS------------------------------------------------------------

/// A struct that holds all the elevators aswell as all active lights
pub struct Worldview{

    orders: Vec<Elevator>,
    lights: Vec<u8>,
}

/// All possible roles of a node
pub enum Role{
    Master,
    Slave,
    Error,
}

//-----------------------FUNCTIONS---------------------------------------------------------


/// Sends an order to a slave elevator and waits for an acknowledgment.
///Broadcast order and wait for responce from reciver, if not recived resend, if this fail. find return false
///The diffrence from just adding from worldview broadcast and from give_order() is that unlike regular udp_broadcast() give_order() requires an acknoledgement from the recivers
/// 
/// # Arguments:
/// 
/// * `master` - &Elevator - A reference to the master `Elevator` initiating the order.
/// * `slave_id` - u8 - The ID of the elevator receiving the order.
/// * `new_order` - u8 - floor number of the new order.
/// 
/// # Returns:
///
/// Returns - bool- `true` if the order was successfully acknowledged, otherwise `false`.
///
fn give_order(master: &Elevator, slave_id: u8, new_order: u8) -> bool {

    let mut retries = 3;
    let message = make_udp_msg(master.id, MessageType::NewOrder, new_order);

    udp_broadcast(&socket, &slave_address(slave_id), &message);

    let mut retry: u8 = 4;
    let mut accepted: Vec<u8> = Vec::new();

    while retry > 0{
        udp_receive(); 
    // add id of ack sender to accepted
        if accepted.len() == elevators.len() {
            return true;
0       }else{
            println!("Missing acknowledgements from active elevators");
        }
        retry -= 1;
    }
    return false;

    println!("Failed to deliver order to slave {}", slave_id);
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
fn remove_from_queue(slave_id: u8, removed_orders: Vec<u8>) -> bool {

    let message = make_udp_msg(master.id, MessageType::RemoveOrder, removed_orders);
    return udp_send_ensure(&socket, &slave_address(slave_id), &message);
}


/// correct_master_worldview
/// Compare message and send out the corrected worldview (union of the recived and current worldview)
/// 
/// # Arguments:
/// 
/// * `master` - &Elevator - Refrence to master elevator.
/// 
/// # Returns:
///
/// Returns - bool- `true` if the order was successfully acknowledged, otherwise `false`.
///
fn correct_master_worldview(master: &Elevator) -> bool {

    let missing_orders = todo!("Vector of vectors containing the queues from the slave with orders that dont exist in worldview");

    // for order in missing_orders
    // union of order and exisiting queue in worldview

    let message = make_udp_msg(master.id, MessageType::Worldview, state);
    
    return udp_broadcast(&socket, &message);
}

/// Broadcast worldview
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
fn master_worldview(master:Elevator) -> bool{
    
    make_Udp_msg(sender_id: master,message_type: Wordview, message:Vec<u8>); 
    
    return udp_broadcast(&socket,&message);
}

// Give away master role, NOT NEEDED, KILL INSTEAD
/*
fn relinquish_master(master: &mut Elevator) -> bool {

    let message = make_udp_msg(master.id, MessageType::RelinquishMaster, vec![]);
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
fn handle_slave_failure(slave_id: u8, elevators: &mut Vec<Elevator>)  -> bool {

    println!("Elevator {} is offline, redistributing elevator {}'s orders.", slave_id,slave_id);

    // Find and redistribute orders for elevator with that spesific ID
    if let Some(index) = elevators.iter().position(|elevator| elevator.ID == slave_id) {
        // Have to use clone to not take ownership of the queue variable(problem compiling)
        let orders = elevators[index].queue.clone(); 
        elevators.remove(index);
        reassign_orders(orders, elevators);
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
/// * `elevators` - &mut Vec<Elevator> - refrence to the vector containing the active elevators.
/// 
/// # Returns:
/// 
/// returns nothing.
///
fn reassign_orders(orders: Vec<u8>)  {

    for order in orders {
        for best_alternative in best_to_worst_elevator(order){
            msg= make_Udp_msg(sender_id:my_id, message_type: message_type, message:Vec<u8>)
            // fix inputs to udp_send_ensure function, dont remember exactly how it was, check udp.rs.
            udp_send_ensure(&UdpSocket, &str, &UdpMsg);
        }
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
fn best_to_worst_elevator(order: u8, elevators: &Vec<Elevator>) -> Vec<u8> {

    // Vec<Elevator.ID, Score> Higher score = better alternative
    let mut scores: Vec<(u8, i32)> = Vec::new(); 


    // Give score to all active elevators
    for elevator in elevators {
        let mut score = 0;

        // Distance to the order (lower is better)
        score -= 10*(elevator.current_floor as i32 - order as i32).abs();

        // Direction compatibility
        if elevator.status == Status::Moving {
            if (elevator.direction == 1 && elevator.current_floor < order) || 
               (elevator.direction == -1 && elevator.current_floor > order) {
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
fn handle_multiple_masters(me: &Elevator, sender: &Elevator, worldview: &Worldview) -> bool {
    
    if me.role == role::Master {
        return false;

        // Give away master role, simple solution, Kill program and reboot
    }else if sender.ID < me.ID{
        reboot_program();

        // Keep master role
    }else{
            return true; 
    }
    
}
