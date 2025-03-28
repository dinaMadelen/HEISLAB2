//! ## Master Module
//! This module provides structs and functions for the master node
//! 
//! ## The structs includes:
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

// use crate::modules::master_functions::master;
#[warn(non_snake_case)]
#[allow(unused_imports)]
#[allow(unused_variables)]

//-----------------------IMPORTS------------------------------------------------------------
use crate::modules::udp_functions::udp::{UdpMsg, UdpData,MessageType,UdpHandler,udp_broadcast,make_udp_msg};
use crate::modules::cab_object::elevator_status_functions::Status;
use crate::modules::cab_object::cab::Cab;
// use crate::modules::slave_functions::slave::reboot_program;
use crate::modules::order_object::order_init::Order;
use crate::modules::system_status::SystemState;
use crate::modules::elevator_object::alias_lib::{CAB, DIRN_DOWN, DIRN_UP};
use crossbeam_channel as cbc;

use serde::{Serialize, Deserialize};
use std::sync::Arc;

//-----------------------STRUCTS------------------------------------------------------------

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
    let mut already_handeld=Vec::new();
    let mut not_handeld=Vec::new();

    // Clone necessary data before dropping mutex lock
    let mut elevator = known_elevators_locked[elevator_index].clone();

    //Check if order is already being handeld
    for order in &new_order {
        if order.order_type != CAB {
            //For all alive elevators
            let alive_elevators: Vec<&Cab> = known_elevators_locked.iter().filter(|e| e.alive).collect();
            for possible_other_server in alive_elevators{
                // Elevator is alive, and has a cabcall or similar order, then we assume the order will be handeld by this elevator
                if possible_other_server.queue.iter().any(|o: &Order| {o.floor == order.floor && (o.order_type == order.order_type)|| order.order_type == CAB}) {
                    already_handeld.push(order.clone());
                }
            }
        }
    }
    
    // Remove orders that are being handeld
    if !already_handeld.is_empty() {
        not_handeld = new_order.into_iter().filter(|o| !already_handeld.contains(o)).collect();
        println!("Order is covered by other orders, ignoring");
    }else{
        not_handeld = new_order;
    }
    
    // Release known_elevators
    drop(known_elevators_locked);
    
    // Add new orders to elevator
    for order in not_handeld {
        elevator.queue.push(order.clone());
        println!("Added order{} to ID:{}",order.floor,elevator.id);
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
/// * `missing_orders` - &Vec<Cab> - refrence to a list of elevators with errors in queues.
/// * `state` - &Arc<SystemState> - Refrence to the systemstate
/// * `udp_handler``- &Arc<UdpHandler> - handler for sending udp messages
/// * `order_update_tx` - &cbc::Sender<Vec<Order>> - crossbeam channel to update lights.
/// 
/// # Returns:
///
/// Returns - bool- `true` if the order was successfully acknowledged, otherwise `false`.
///
pub fn correct_master_worldview(discrepancy_cabs:&Vec<Cab>, state: &Arc<SystemState>,udp_handler: &Arc<UdpHandler>, order_update_tx: &cbc::Sender<Vec<Order>>) -> bool {
    println!("Correcting worldview for master");

    let mut changes_made = false;

    if discrepancy_cabs.is_empty(){
        println!("List of missing cabs is empty");
        return false;
    }

    // Compare elevators to missing orders list
    let mut known_elevators_locked = state.known_elevators.lock().unwrap().clone();
    for missing_elevator in discrepancy_cabs.iter() {
        let mut known_elevators_locked = state.known_elevators.lock().unwrap();
        //Add missing orders to cabs
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
        // Add missing orders to all orders 
        let mut all_orders_locked = state.all_orders.lock().unwrap();
        for order in &missing_elevator.queue {
            if !all_orders_locked.contains(order) {
                all_orders_locked.push(order.clone());
            }
                println!("Added missing orders from {} to state.all_orders", missing_elevator.id);
        }

        
        let known_elevators_locked = state.known_elevators.lock().unwrap();
        // Collect all known elevator orders into a flat Vec
        let distributed_orders: Vec<Order> = known_elevators_locked
            .iter()
            .flat_map(|cab| cab.queue.iter().cloned())
            .collect();
        
       //Swap mutex
        drop(known_elevators_locked);
        all_orders_locked = state.all_orders.lock().unwrap();

        
        // Find orders in all_orders that are NOT in distributed orders
        let missing_orders: Vec<Order> = all_orders_locked
            .iter()
            .filter(|order| !distributed_orders.contains(order))
            .cloned()
            .collect();
        
        drop(all_orders_locked);

        reassign_orders(&missing_orders,state,udp_handler,order_update_tx);
    }

    return changes_made;
}



/// master_worldview
/// Compare message and send out the corrected worldview (union of the recived and current worldview)
/// 
/// # Arguments:
/// 
/// * `state` - SystemState - Refrence to state of the system struct see SystemState).
/// * `UdpHandler` - Handler for sending udp messages. 
/// 
/// # Returns:
///
/// Returns - bool- `true` if the order was successfully broadcasted, otherwise `false`.
///
pub fn master_worldview(state:&Arc<SystemState>, udphandler: &Arc<UdpHandler>) -> bool{

    println!("Starting worldview");

    let known_cabs = state.known_elevators.lock().unwrap().clone();
    

    let worldview_msg = make_udp_msg(state.me_id, MessageType::Worldview, UdpData::Cabs(known_cabs.clone()));
    for elevator in known_cabs.iter(){
        udphandler.send(&elevator.inn_address, &worldview_msg);
    }
    println!("preparing to broadcast");
    let message = make_udp_msg(state.me_id, MessageType::Worldview, UdpData::Cabs(known_cabs)); 
    return udp_broadcast(&message);
}

/// handle_slave_failure
/// Handle slave failure, take action to secure service for orders when a slave goes offline
/// 
/// # Arguments:
/// 
/// * `slave_id` - u8 - ID of the elevator that has failed.
/// * `elevators` - &mut Vec<Cab> - refrence to the vector containing the active elevators.
/// * `state` - &mut SystemState - mutable refrence to the systemstate
/// * `udp_handler` - refrence to the handler that should send message.
/// * `order_update_tx` -cbc::Sender<Vec<Order>>)-  crossbeam channel to notify other threads
/// 
/// # Returns: 
/// 
/// Returns - bool - `true` if orders have been succsessfully distributed and elevator har been removed from active elevators
/// else returns `false`
///
pub fn handle_slave_failure(slave_id: u8, elevators: &mut Vec<Cab>,state: &Arc<SystemState>, udp_handler: &UdpHandler, order_update_tx: cbc::Sender<Vec<Order>>)  -> bool {

    println!("Elevator {} is offline, redistributing elevator {}'s orders.", slave_id,slave_id);

    return reassign_elevator_orders(slave_id, state, &udp_handler, order_update_tx.clone());
}

/// reassign orders
/// Reassigns a or more orders from one elevator to active elevators
/// it also tries to reassign messages that currently have no dedicated elevator
///  
/// # Arguments:
/// 
/// * `orders` - &Vec<u8> - refrence to list of orders to be distributed.
/// * `state` - mutable refrence to system state
/// * `udp_handler` - &UdpHandler> - refrence to the handler that should send the order.
/// * `order_update_tx` - &order_update_tx-  crossbeamchannel for messagepassing between threads.
/// 
/// # Returns:
/// 
/// returns `true`, if successfull and `false` if failed.
///
pub fn reassign_orders(orders: &Vec<Order>, state: &Arc<SystemState>, udp_handler: &Arc<UdpHandler>, order_update_tx: &cbc::Sender<Vec<Order>>) -> bool {
    
    // Copy value in mutexes
    let mut all_orders = state.all_orders.lock().unwrap().clone();
    let mut known_elevators = state.known_elevators.lock().unwrap().clone();

    // Find all orders currently assigned to any elevator
    let all_assigned_orders: Vec<Order> = known_elevators.iter().flat_map(|e| e.queue.iter().cloned()).collect();

    // Filter out orders that are already in any elevator's queue
    let mut missing_orders = all_orders.iter().filter(|o| !all_assigned_orders.contains(o)).cloned().collect();

    let mut combined_orders = orders.clone();
    combined_orders.append(&mut missing_orders);

    for order in combined_orders {
        
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


/// Reassign elevator orders
/// Reassigns all non cab calls from one cab, distributed amongst all active active
///  
/// # Arguments:
/// 
/// * `error_cab_id` - u8 - id of the elevator that who's orders are being reassigned
/// * `state` - mutable refrence to system state
/// * `udp_handler` - &UdpHandler> - refrence to the handler that should send the order.
/// * `order_update_tx` - &order_update_tx-  crossbeamchannel for messagepassing between threads.
/// 
/// # Returns:
/// 
/// returns `true`, if successfull and `false` if failed.
///
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
/// best to worst elevator
/// Cost function that returns order to the best fitting elevators from best to worst alternative.
///  
/// # Arguments:
/// 
/// * `order` - &Order - refrence to the order.
/// * `elevators` - &Vec<Cab> - refrence to list of active elevators that the functions will sort.
/// 
/// # Returns:
///
/// Retruns - Vec<u8> - a list of IDs in decending order from best fit to worst fit.
///
pub fn best_to_worst_elevator(order: &Order, elevators: &Vec<Cab>) -> Vec<u8> {
    let mut scores: Vec<(u8, i32)> = Vec::new();
    for elevator in elevators {
        let mut score = 0;

        // Distance: closer floors get a higher score.
        let distance = (elevator.current_floor as i32 - order.floor as i32).abs();
        score -= 10 * distance;

        // Direction compatibility: reward if moving in the right direction.
        if elevator.status == Status::Moving {
            if (elevator.direction == DIRN_UP && elevator.current_floor < order.floor)
                || (elevator.direction == DIRN_DOWN && elevator.current_floor > order.floor)
            {
                score += 10;
            } else {
                score -= 10;
            }
        } else if elevator.status == Status::Idle {
            // Idle elevators are preferred.
            score += 30;
        } else if elevator.status == Status::Error {
            score -= 10000;
        }

        // If elevator is not alive, heavy penalty.
        if !elevator.alive {
            score -= 20000;
        }
        // Shorter queue gets priority.
        score -= 10 * elevator.queue.len() as i32;

        scores.push((elevator.id, score));
    }

    // Sort in descending order 
    scores.sort_by(|a, b| b.1.cmp(&a.1));

    // Return only the elevator IDs in sorted order.
    scores.into_iter().map(|(id, _)| id).collect()
}

/// fix master issues
/// If for some reason more than master is active, forexample race during election or one didnt recive the first message from new master.
/// master with lowest ID keeps the role, the rest become slaves.
/// 
/// # Arguments:
/// 
/// * `stae` - &mut SystemState - mutable refrence to the systemstate.
/// * `udp_handlerr` - &u8 - refrence to udp handler to send udp messages.
/// 
/// # Returns:
///
/// Returns - Some(bool) - returns false if the ID is its own, returns true if it keeps the master if the ID is higher than the sender, reboots if it is lower.
///
pub fn fix_master_issues(state: &Arc<SystemState>, udp_handler: &UdpHandler) {
    // Make lowest id alive the master id
    {
        // Lock master_id first.
        let old_master_id = state.master_id.lock().unwrap().clone();
        // Then lock known_elevators.
        let mut known_elevators = state.known_elevators.lock().unwrap();
        // Gather mutable references to all elevators marked as Master.
        let mut masters: Vec<&mut Cab> = known_elevators
            .iter_mut()
            .filter(|cab| cab.role == Role::Master)
            .collect();

        if masters.len() > 1 {
            // Multiple masters found.
            masters.sort_by_key(|cab| cab.id);
            let chosen_master = &masters[0];
            let chosen_master_id = chosen_master.id;
            println!(
                "Multiple masters detected. Keeping elevator {} as master.",
                chosen_master_id
            );

            // Set the shared master id.
            let mut master_id_guard = state.master_id.lock().unwrap();
            *master_id_guard = chosen_master_id;
            drop(master_id_guard);

            // Reassign all other masters to slave.
            for cab in masters.iter_mut().skip(1) {
                println!("Reassigning elevator {} from master to slave.", cab.id);
                cab.role = Role::Slave;
            }
        } else if masters.is_empty() {
            // No elevator is master.
            println!("No masters alive, setting new master.");
            // Find all alive elevators.
            let mut alive_elevators: Vec<&mut Cab> = known_elevators
                .iter_mut()
                .filter(|cab| cab.alive)
                .collect();
            alive_elevators.sort_by_key(|cab| cab.id);
            if let Some(new_master) = alive_elevators.first_mut() {
                new_master.role = Role::Master;
                let mut master_id_guard = state.master_id.lock().unwrap();
                *master_id_guard = new_master.id;
            }
        } else {
            // Exactly one master exists.
            println!("No multiple-master conflict detected.");
        }

        let master_id_guard = state.master_id.lock().unwrap();
        if !(old_master_id == *master_id_guard){
            if let Some(master_elevator) = known_elevators.iter().find(|cab| cab.id == *master_id_guard)
            {
                let msg = make_udp_msg(
                    state.me_id,
                    MessageType::NewMaster,
                    UdpData::Cab(master_elevator.clone()),
                );
                for elevator in known_elevators.iter() {
                    udp_handler.send(&elevator.inn_address, &msg);
                }
            }
        }
    }

}
