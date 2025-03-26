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
#[allow(unused_imports)]
#[allow(unused_variables)]

//-----------------------IMPORTS------------------------------------------------------------
use crate::modules::udp_functions::udp::{UdpMsg, UdpData,MessageType,UdpHandler,udp_broadcast,make_udp_msg};
use crate::modules::cab_object::elevator_status_functions::Status;
use crate::modules::cab_object::cab::Cab;
use crate::modules::slave_functions::slave::reboot_program;
use crate::modules::order_object::order_init::Order;
use crate::modules::system_status::SystemState;
use crate::modules::elevator_object::alias_lib::{CAB,DIRN_DOWN, DIRN_STOP};
use crossbeam_channel as cbc;


use serde::{Serialize, Deserialize};
use std::sync::Arc;

/* 
//-----------------------GLOBAL VARIABLES---------------------------------------------------
static mut FAILED_ORDERS: Option<Arc<Mutex<Vec<Order>>>> = None;
*/


//-----------------------STRUCTS------------------------------------------------------------

/*  Not used
/// A struct that holds all the active elevators aswell as all active lights
pub struct Worldview{
    elevators: Vec<Cab>,
    lights: Vec<u8>,
}
*/
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
///  * `elevator_id` - u8 - ID of the elevator that the order should be added too
///  * `new_order` - Vec<&Order> - List of refrences to orders that should be added.
///  * `state` - &mut SystemState - Mutable refrence to the state of the system.
///  * `udp_handler` - &UdpHandler - refrence to the handler that should handle the sending
/// 
/// # Returns:
///
/// Returns - bool- `true` if the order was successfully acknowledged, otherwise `false`.
///
/// 
pub fn give_order(elevator_id: u8, new_order: Vec<&Order>, state: &Arc<SystemState>, udp_handler: &UdpHandler) -> bool {

    println!("Give order entered");

    // Lock known_elevators
    let known_elevators_locked = state.known_elevators.lock().unwrap();
    // Find the elevator and copy the needed data
    let elevator_index = match known_elevators_locked.iter().position(|e| e.id == elevator_id && e.alive) {
        Some(index) => index,
        None => {
            println!("ERROR: Elevator ID {} not found in active elevators.", elevator_id);
            return false;
        }
    };

    // Clone necessary data before dropping mutex lock
    let mut elevator = known_elevators_locked[elevator_index].clone();

    // Release known_elevators
    drop(known_elevators_locked);
    
    // Add new orders to elevator
    for order in new_order {
        elevator.queue.push(order.clone());
    }

    // Inform rest of system that the order has been added
    let message = make_udp_msg(state.me_id,MessageType::NewOrder, UdpData::Cab(elevator.clone()));
    println!("Broadcasting new orders for elevator:{}", elevator.id);

    // Broadcast message
    return udp_handler.ensure_broadcast(&message,state,5);
}



/// correct_master_worldview
/// Compare message and send out the corrected worldview (union of the recived and current worldview)
/// 
/// # Arguments:
/// 
/// * `missing_orders` - Vec<Cab> - List of elevators with errors in queues.
/// * `known_elevators` - &mut Vec<Cab> - Refrence to master elevator.
/// 
/// # Returns:
///
/// Returns - bool- `true` if the order was successfully acknowledged, otherwise `false`.
///
pub fn correct_master_worldview(discrepancy_cabs:&Vec<Cab>, state: &Arc<SystemState>) -> bool {
    println!("Correcting worldview for master");

    let mut changes_made = false;

    if discrepancy_cabs.is_empty(){
        println!("List of missing cabs is empty");
        return false;
    }


    // Compare elevators to missing orders list
    let mut known_elevators_locked = state.known_elevators.lock().unwrap();
    for missing_elevator in discrepancy_cabs.iter() {
        if let Some(elevator) = known_elevators_locked.iter_mut().find(|e| e.id == missing_elevator.id) {
            for order in &missing_elevator.queue {
                if !elevator.queue.contains(&order) {
                    elevator.queue.push(order.clone());
                    println!("Added missing order {:?} to elevator {}", order.floor, elevator.id);
                    changes_made = true;
                }
            }
        } else {
            println!(
                "Warning: Elevator ID {} from missing_orders not found in active elevators",
                missing_elevator.id
            );
        }
    }
    drop(known_elevators_locked);

    return changes_made;
}


/// generate_worldview
/// Create worldview from list of active elevators
/// finds active lights from orders of active elevators
/// 
/// # Arguments:
/// 
/// * `known_elevators` - &Vec<Cab> - Refrence to list of active elevators
/// 
/// # Returns:
///
/// Returns - Worldview- Returns a worldview struct.
///


/*Not used
pub fn generate_worldview(known_elevators: &Vec<Cab>) -> Worldview {

    // Find active lights
    let mut lights = Vec::new();
    for elevator in known_elevators {

        for order in &elevator.queue {
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
        elevators: known_elevators.clone(), 
        lights,                       
    };
}
*/


/// master_worldview
/// Compare message and send out the corrected worldview (union of the recived and current worldview)
/// 
/// # Arguments:
/// 
/// * `state` - SsytemState - Refrence to list of active elevators(mutex, see SysState).
/// 
/// # Returns:
///
/// Returns - bool- `true` if the order was successfully broadcasted, otherwise `false`.
///
pub fn master_worldview(state:&Arc<SystemState>) -> bool{

    println!("Starting worldview");

    let known_cabs = state.known_elevators.lock().unwrap().clone();
    
    println!("preparing to send");
    let message = make_udp_msg(state.me_id, MessageType::Worldview, UdpData::Cabs(known_cabs)); 
    return udp_broadcast(&message);
}

// Give away master role, NOT NEEDED, KILL INSTEAD
/*
fn relinquish_master(master: &mut Cab) -> bool {

    let message = make_udp_msg(master.ID, MessageType::RelinquishMaster, vec![]);
    udp_broadcast(&socket, &message);

    master.status = Cab.status::Error;
    return true;
}
*/


/// handle_slave_failure
/// Handle slave failure, take action to secure service for orders when a slave goes offline
/// 
/// # Arguments:
/// 
/// * `slave_id` - u8 - ID of the elevator that has failed.
/// * `elevators` - &mut Vec<Cab> - refrence to the vector containing the active elevators.
/// * `state` - &mut SystemState - mutable refrence to the systemstate
/// * `udp_handler` - refrence to the handler that should send message.
/// 
/// # Returns: 
/// 
/// Returns - bool - `true` if orders have been succsessfully distributed and elevator har been removed from active elevators
/// else returns `false`
///
///
pub fn handle_slave_failure(slave_id: u8, elevators: &mut Vec<Cab>,state: &Arc<SystemState>, udp_handler: &UdpHandler, order_update_tx: cbc::Sender<Vec<Order>>)  -> bool {

    println!("Elevator {} is offline, redistributing elevator {}'s orders.", slave_id,slave_id);

    return reassign_elevator_orders(slave_id, state, &udp_handler, order_update_tx.clone());
}



/// Reassign failed orders
/// Reassigns a or more orders from one elevator to active elevators
///  
/// # Arguments:
/// 
/// * `orders` - &Vec<u8> - refrence to list of orders to be distributed.
/// * `state` - mutable refrence to system state
/// * `udp_handler` - &UdpHandler> - refrence to the handler that should send the order.
/// 
/// # Returns:
/// 
/// returns `true`, if successfull and `false` if failed.
///
pub fn reassign_orders(orders: &Vec<Order>, state: &Arc<SystemState>, udp_handler: &UdpHandler, order_update_tx: cbc::Sender<Vec<Order>>) -> bool {
    for order in orders {
        
        let mut assigned = false;

        if order.order_type != CAB { 

            //Lock active elevators and copy, then release
            let elevators= state.known_elevators.lock().unwrap().clone();

            //Give order to best alternative
            for best_alternative in best_to_worst_elevator(&order, &elevators) {
                println!("Assigning order {} to elevator {}", order.floor, best_alternative);

                if give_order(best_alternative, vec![&order],state,udp_handler) {
                    println!("Order {} successfully reassigned to elevator {}", order.floor, best_alternative);
                    assigned = true;
                    break; 
                } else {
                    println!("Failed to assign order {} to elevator {}. Trying next option", order.floor, best_alternative);
                }
            }
        }
        
        let mut failed_orders_locked = state.all_orders.lock().unwrap();

        // If no elevator accepted the order, store it for retry
        if !assigned{
            println!("No available elevator for order {}. Storing to retry later.", order.floor);
            failed_orders_locked.push(order.clone());
            drop(failed_orders_locked);
        }
    }

    let failed_orders_locked = state.all_orders.lock().unwrap();

    if failed_orders_locked.is_empty() {
        println!("All failed orders are redistributed");
        return true;
    } else {
        println!("There are failed to be distributed");
        return false;
    }
}


pub fn reassign_elevator_orders(error_cab_id: u8 , state: &Arc<SystemState>, udp_handler: &UdpHandler, order_update_tx: cbc::Sender<Vec<Order>>) -> bool {
    
    //Changing elevator from active elevators to inactive
    // Remove from active queue
    let mut known_elevators_locked = state.known_elevators.lock().unwrap();
    if let Some(elevator) = known_elevators_locked.iter_mut().find(|e| e.id == error_cab_id) {
        elevator.alive = false;
        println!("Set elevator ID:{} as offline.", error_cab_id);
    } else {
        println!("Error: can't find elevator ID {}, in known list", error_cab_id);
    }
    

    //Find this elevator
    if let Some(elevator) = known_elevators_locked.iter_mut().find(|e| e.id == error_cab_id) {
        let mut assigned = true;

        //For each order check if CAB order
        for order in elevator.queue.clone(){

            // Do not reassign CAB orders
            if order.order_type != CAB { 

                //Lock known elevators and copy the active ones, then release
                let live_elevators: Vec<_> = state.known_elevators.lock().unwrap().clone().into_iter().filter(|e| e.alive).collect();

                //Give order to best alternative
                for best_alternative in best_to_worst_elevator(&order, &live_elevators) {
                    println!("Assigning order {} to elevator {}", order.floor, best_alternative);

                    // Give the order to the best alternative and remove the order from the dead elevator
                    if give_order(best_alternative, vec![&order],state,udp_handler) {

                        //lock mutex
                        let mut known_elevators_locked=state.known_elevators.lock().unwrap();

                        //Find only the one that match ID
                        if let Some(real_elevator) = known_elevators_locked.iter_mut().find(|e| e.id == error_cab_id) {
                            real_elevator.queue.retain(|o| *o != order);
                            println!("Order removed from ID:{} and succsesfully redistributed", error_cab_id);
                        } else {
                            println!("Could not find elevator with ID {} in dead_elevators", error_cab_id);
                        }
                        drop(known_elevators_locked);
                        break; 
                    
                    } else {
                        println!("Failed to assign order {} to elevator {}. Trying next option", order.floor, best_alternative);
                        assigned = false;
                    }
                }

                let mut failed_orders_locked = state.all_orders.lock().unwrap();
                // If no elevator accepted the order, store it for retry
                if !assigned{
                    println!("No available elevator for order {}. Storing to retry later.", order.floor);
                    failed_orders_locked.push(order.clone());
                    drop(failed_orders_locked);
                } 
            }
        }
    }else{
        println!("Couldnt find ID:{}, in active elevators", error_cab_id);
        }


    

    let failed_orders_locked = state.all_orders.lock().unwrap();
    if failed_orders_locked.is_empty() {
        println!("All failed orders are redistributed");
        return true;
    } else {
        println!("There are failed to be distributed");
        return false;
    }
    
}

/// Cost function that returns order to the best fitting elevators from best to worst alternative.
///  
/// # Arguments:
/// 
/// * `order` - &Order - refrence to the order.
/// * `elevators` - &Vec<Cab> - refrence to list of active elevators that the functions will sort.
/// 
/// # Returns:
///
/// Retruns - Vec<u8> - a list of i IDs in decending order from best fit to worst fit.
///
pub fn best_to_worst_elevator(order: &Order, elevators: &Vec<Cab>) -> Vec<u8> {

    // Vec<Cab.ID, Score> Higher score = better alternative
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
        if elevator.alive == false{
            score -= 20000;
        }

        // Shorter queue gets priority, Less is better
        score -= elevator.queue.len() as i32 * 5; 

        scores.push((elevator.id, score));
    }

    // Sort by score
    scores.sort_by(|a, b| b.1.cmp(&a.1));

    // Return Vec<u8> of IDs in decending order from best to worst option  https://doc.rust-lang.org/std/iter/struct.Map.html
    return scores.into_iter().map(|(id, _score)| id).collect();
}

/// handle_multiple_masters
/// If for some reason more than master is active, forexample race during election or one didnt recive the first message from new master.
/// master with lowest ID keeps the role, the rest become slaves.
/// 
/// # Arguments:
/// 
/// * `stae` - &mut SystemState - mutable refrence to the systemstate.
/// * `sender` - &u8 - refrence to the ID of the master it is comparing to.
/// 
/// # Returns:
///
/// Returns - Some(bool) - returns false if the ID is its own, returns true if it keeps the master if the ID is higher than the sender, reboots if it is lower.
///
pub fn handle_multiple_masters(state: &Arc<SystemState>, sender: &u8) -> bool {

    if state.me_id == state.master_id.lock().unwrap().clone(){
        
        // Lock active elevators
        let mut known_elevators_locked = state.known_elevators.lock().unwrap(); 

        // Confirm elevator is active
        let me = match known_elevators_locked.iter_mut().find(|e| e.id == state.me_id && e.alive){

        Some(me_elevator) => me_elevator,

        None => {println!("ERROR: ID {} is known or not alive", state.me_id);
        return false;
    }
};

        let mut result = true;
        
            //Master ID is my ID
        if me.role == Role::Master {
            result = false; 

            // Give away master role, Kill program and reboot
        }else if sender < &me.id{
            reboot_program();
        } 
        return result;
    }

    return false; 
}
