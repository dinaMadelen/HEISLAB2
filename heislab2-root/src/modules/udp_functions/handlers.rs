#[allow(unused_imports)]
#[allow(unused_variables)]
#[allow(non_camel_case_types)]

//----------------------------------------------Imports
use std::net::{IpAddr,SocketAddr, UdpSocket};
//use std::ops::DerefMut;                        // https://doc.rust-lang.org/std/net/struct.UdpSocket.html       
use serde::{Deserialize, Serialize};            // https://serde.rs/impl-serialize.html         //Add to Cargo.toml file, Check comment above
                                                // https://docs.rs/serde/latest/serde/ser/trait.Serialize.html#tymethod.serialize
use std::time::{Instant, SystemTime};              // https://doc.rust-lang.org/std/time/struct.Duration.html
// use std::thread::sleep;                      // https://doc.rust-lang.org/std/thread/fn.sleep.html
use std::sync::Arc;                     // https://doc.rust-lang.org/std/sync/struct.Mutex.html
use crossbeam_channel as cbc;


use crate::modules::order_object::order_init::Order;
use crate::modules::elevator_object::elevator_init::SystemState;
use crate::modules::cab_object::cab::Cab;
use crate::modules::master_functions::master::{give_order, best_to_worst_elevator,fix_master_issues,Role,correct_master_worldview, reassign_orders};
use crate::modules::slave_functions::slave::{update_from_worldview, check_master_failure, set_new_master};
use crate::modules::udp_functions::udp::{MessageType, UdpHeader, UdpData, UdpHandler, UdpMsg, udp_ack};


pub use crate::modules::elevator_object::*;
pub use elevator_init::Elevator;
pub use alias_lib::{HALL_DOWN, HALL_UP,CAB, DIRN_DOWN, DIRN_UP, DIRN_STOP};

/// handle_worldview
/// # Arguments:
/// 
/// * `state` - &mut SystemState - mutable refrence to system state.
/// * `msg` - UdpMsg - Recived message.
/// 
/// # Returns:
///
/// Returns - None - .
///
pub fn handle_worldview(state: Arc<SystemState>, msg: &UdpMsg,udp_handler: Arc<UdpHandler>) {

    // My worldview
    if state.me_id == msg.header.sender_id{
        return;
    }

    println!("Updating worldview...");

    //Update last lifesign and last worldview
    let mut last_lifesign_locked = state.lifesign_master.lock().unwrap();
    *last_lifesign_locked = Instant::now();
    drop(last_lifesign_locked);

    let mut new_worldview = state.last_worldview.lock().unwrap();
    *new_worldview = msg.clone();
    drop(new_worldview);
    
    let worldview = if let UdpData::Cabs(worldview) = &msg.data{
        worldview
    }
    else{
        println!("Wrong data in message for worldview");
        return;
    };

    update_from_worldview(&state, &worldview,udp_handler);
    /* let known_elevators: Vec<Cab> = {
    let known_elevators_locked = state.known_elevators.lock().unwrap();
    known_elevators_locked.clone() */ 
     
    //not used
    //generate_worldview(&known_elevators);
}

/// handle_ack
/// 
/// # Arguments:
/// 
/// * `msg` - UdpMsg - Recived message .
/// * `sent_messages` &mut Arc<Mutex<Vec<UdpMsg>>>-  -.
/// 
/// # Returns:
///
/// Returns -None- .
///
pub fn handle_ack(msg: &UdpMsg, state: Arc<SystemState>) {
    
    let sender_id = msg.header.sender_id;
    let original_checksum = if let UdpData::Checksum(original_checksum) = &msg.data {
        *original_checksum
    } else {
        println!("Expected UdpData as Checksum in message, got somthing else");
        return;
    };

    //Lock mutex for messages awaiting response
    let mut sent_messages_locked = state.sent_messages.lock().unwrap();

    if let Some(waiting) = sent_messages_locked.iter_mut().find(|e| e.message_hash == original_checksum){
        // Add sender id if not in responded
        if !waiting.responded_ids.contains(&sender_id){
            waiting.responded_ids.push(sender_id);
            println!("Added Ack from:{:?} for checksum: {:?}", sender_id, original_checksum);
        }

        // Variable to control if all elevators have acked
        let mut all_confirmed = true;

        // Lock mutex for active elevators
        let known_elevators_locked = state.known_elevators.lock().unwrap();

        //Check that all active elevatos have responded 
        for elevator in known_elevators_locked.iter().filter(|e|e.alive){
            if !waiting.responded_ids.contains(&elevator.id){
                println!("Still missing confirmations for elevaotr ID:{}", elevator.id);
                all_confirmed = false;
            }

        }
        drop(known_elevators_locked);

        if all_confirmed{
            waiting.all_confirmed = true;
            println!("Added Ack from:{:?} for checksum:{:?}", sender_id, original_checksum);
        }

        println!("All elevators have confirmed reciving message with checksum: {:?}",original_checksum);
        
    }else {

    println!("Checksum: {:?} not found in list waiting for confirmation, sender was {:?}", original_checksum, sender_id);
    };

}


///handle_nack
/// 
/// # Arguments:
/// 
/// * `msg` - UdpMsg - .The recived message
/// * `sent_messages` - &Arc<Mutex<Vec<UdpMsg>>> - List of messages waiting for responses
/// * `target_adress` - &SocketAddr - refrence to reciver adress .
/// * `udp_handler` - &UdpHandler - refrence to the handler sending the message
/// 
/// # Returns:
///
/// Returns - - .
///
pub fn handle_nak(msg: &UdpMsg, state: Arc<SystemState>, target_address: &SocketAddr,udp_handler: Arc<UdpHandler>) {
    println!("Received NAK from ID: {}", msg.header.sender_id);

    let original_checksum = if let UdpData::Checksum(original_checksum) = &msg.data {
        *original_checksum
    } else {
        println!("Expected UdpData as Checksum in message, got somthing else");
        return;
    };

    // Check if this NAK matches sent message
    let sent_messages_locked = state.sent_messages.lock().unwrap();
    if let Some(waiting_response) = sent_messages_locked.iter().position(|m| m.message_hash == original_checksum) {
        println!("NAK matches message with checksum: {:?}", original_checksum);
    } else {
        println!("ERROR: Received NAK with unknown checksum {:?}", original_checksum);
    }

}



/// handle_new_order
/// # Arguments:
/// 
/// * `msg` - &UdpMsg - refrence to recived message.
/// * `sender_adress` - &SocketAdress - Reciving adress.
/// * `state` - &mut SystemState - Mutable refrence to systemstate
/// * `udp_handler` - &UdpHandler - refrence to handler, sending message.
/// 
///
/// # Returns:
///
/// Returns -None- .
///

pub fn handle_new_order(msg: &UdpMsg, sender_address: &SocketAddr, state: Arc<SystemState>,udp_handler: Arc<UdpHandler>, light_update_tx: cbc::Sender<Vec<Order>>, order_update_tx: cbc::Sender<Vec<Order>>) -> bool {
    println!("New order received ID: {}", msg.header.sender_id);

    let elevator = if let UdpData::Cab(cab) = &msg.data {
        cab
    } else {
        println!("ERROR: Wrong UdpData type for NewOrder");
        return false;
    };

    let elevator_id = elevator.id;

    //Lock active elevators
    let mut known_elevators_locked = state.known_elevators.lock().unwrap(); 

    //Find elevator with mathcing ID and update queue
    if let Some(update_elevator) = known_elevators_locked.iter_mut().find(|e| e.id == elevator_id){
        for order in &elevator.queue {
            if !update_elevator.queue.contains(&order){
                update_elevator.queue.push(order.clone());
                println!("Order {:?} successfully added to elevator {}.", order, elevator.id);
                light_update_tx.send(update_elevator.queue.clone()).unwrap();
                order_update_tx.send(vec![order.clone()]).unwrap();

            }else {
                println!("Order {:?} already in queue for elevator {}.", order, elevator.id);
            }
        }
    }
    for elevator in known_elevators_locked.iter(){
        udp_ack(*&elevator.inn_address, &msg, elevator.id, &udp_handler);
    }
    //Send Ack to sender
    return udp_ack(*&elevator.inn_address, &msg, elevator.id, &udp_handler);
}

/// handle_new_master
/// # Arguments:
/// 
/// * `msg` - UdpMsg - recived message.
/// * `known_elevators` - &Arc<Mutex<Vec<Cab>>> - Vector of active elevators.
/// 
/// # Returns:
///
/// Returns -None- .
///
pub fn handle_new_master(msg: &UdpMsg, state: Arc<SystemState>) {
    let master_id = state.master_id.lock().unwrap().clone();
    let known_master_id = state.master_id.lock().unwrap().clone();
    if  !(known_master_id == master_id){
        println!("New master detected, ID: {}", msg.header.sender_id);

    let cab_to_be_master = if let UdpData::Cab(cab) = &msg.data{
        cab.clone()
    }else{
        println!("Couldnt read OrderComplete message");
        return;
    };

    // Set current master's role to Slave
    let mut known_elevators_locked = state.known_elevators.lock().unwrap();
    if let Some(current_master) = known_elevators_locked.iter_mut().find(|elevator| elevator.role == Role::Master) {
        println!("Changing current master (ID: {}) to slave.", current_master.id);
        current_master.role = Role::Slave;
    } else {
        println!("ERROR: No active master found.");
    }

    // Set new master
    if let Some(new_master) = known_elevators_locked.iter_mut().find(|elevator| elevator.id == cab_to_be_master.id) {
        println!("Updating elevator ID {} to Master.", cab_to_be_master.id);
        new_master.role = Role::Master;

        let mut master_id = state.master_id.lock().unwrap();
        *master_id = cab_to_be_master.id;
        drop(master_id);

    } else {
        println!("Error: Elevator ID {} not found in active list.", msg.header.sender_id);
    }
    }
    
}

/// handle_new_online
/// Adds a new online elevator to the active elevators vector.
/// # Arguments:
/// 
/// * `msg` - `&UdpMsg` - Message received containing the elevator ID.
/// * `state` - `&mut System State>` - mutable refrence to systemstate.
/// 
/// # Returns:
///
/// Returns `true` if the elevator was added or already in the vector, otherwise `false`.
///
pub fn handle_new_online(msg: &UdpMsg, state: Arc<SystemState>) -> bool {
    println!("New elevator online, ID: {}", msg.header.sender_id);

    //Lock active elevaotrs
    let mut known_elevators_locked = state.known_elevators.lock().unwrap();

    // Check if elevator is already active
    if known_elevators_locked.iter().any(|e| e.id == msg.header.sender_id && e.alive) {
  
        println!("Elevator ID:{} is already active.", msg.header.sender_id);
        return true;
    }else if let Some(cab) = known_elevators_locked.iter_mut().find(|e| e.id == msg.header.sender_id && !e.alive){
        cab.alive=true;
        println!("Elevator ID: is set alive, already known elevator");
        return true;
    }

    //Release active elevators
    drop(known_elevators_locked); 
    
    let msg_elevator = if let UdpData::Cab(cab) = &msg.data {
        cab
    } else {
        println!("Error: Wrong UdpData for message type");
        return false;
    };

    println!("New unknown elevator online ID: {}, adding to known", msg.header.sender_id);

    // Create new elevator
    let new_elevator = Cab {
        inn_address: msg_elevator.inn_address,
        out_address: msg_elevator.out_address,
        num_floors: msg_elevator.num_floors,
        id: msg_elevator.id,
        current_floor: msg_elevator.current_floor,
        last_served_floor: msg_elevator.last_served_floor,
        queue: msg_elevator.queue.clone(),
        status: msg_elevator.status.clone(),
        direction: msg_elevator.direction.clone(),
        role: msg_elevator.role.clone(),
        last_lifesign: SystemTime::now(),
        alive: true
    };

    // Lock again and add the new elevator
    let mut known_elevators_locked = state.known_elevators.lock().unwrap();
    known_elevators_locked.push(new_elevator);
    drop(known_elevators_locked); 

    println!("Added new elevator ID {}.", msg.header.sender_id);
    return true;
}

/// handle_error_worldview
/// # Arguments:
/// 
/// * `msg` - &UdpMsg - refrence to UDP message.
/// * `known_elevators` - &Arc<Mutex<Vec<Cab>>> - List of active elevators .
/// 
/// # Returns:
///
/// Returns -None- .
///
pub fn handle_error_worldview(msg: &UdpMsg, state: Arc<SystemState>,udp_handler: &Arc<UdpHandler>, order_update_tx: &cbc::Sender<Vec<Order>>) {
    println!("EROR: Worldview error reported by ID: {}", msg.header.sender_id);

    // List of orders from sender
    let mut missing_orders = if let UdpData::Cabs(worldview) = &msg.data {
        worldview.clone()
    } else {
        println!("ERROR: Expected UdpData::Cabs but got something else");
        return;
    };

    // Compare and correct worldview based on received data
    if correct_master_worldview(&mut missing_orders, &state, udp_handler, &order_update_tx) {
        println!("Worldview corrected based on report from ID: {}", msg.header.sender_id);
    } else {
        println!("ERROR: Failed to correct worldview");
    }
}

/// handle_error_offline
/// # Arguments:
/// 
/// * `msg` - &UdpMsg - refrence to UDP message.
/// * `state` - &mut SystemState - mutable refrence to the system state.
/// * `udp_handler` - &UdpHandler - refrence to the handler sending the message.
/// 
/// # Returns:
///
/// Returns - None - .
///
pub fn handle_error_offline(
    msg: &UdpMsg,
    state: Arc<SystemState>,
    udp_handler: &Arc<UdpHandler>,
    order_update_tx: &cbc::Sender<Vec<Order>>,
) {
    

    if let UdpData::Cab(ref cab) = msg.data {
        // Update the shared state directly without cloning.
        println!("Elevator {} went offline. Reassigning orders", cab.id);

        let mut known_elevators = state.known_elevators.lock().unwrap();
        if let Some(elevator) = known_elevators.iter_mut().find(|e| e.id == cab.id) {
            elevator.alive = false;
            elevator.role = Role::Slave;
            println!("Elevator ID:{} set to offline.", cab.id);
        } else {
            println!("Elevator ID:{} not found in known elevators.", cab.id);
            return;
        }
        drop(known_elevators);
        //

        // Check if the offline elevator was the master.
        let was_master = {
            let known_elevators = state.known_elevators.lock().unwrap();
            if let Some(elevator) = known_elevators.iter().find(|e| e.id == cab.id) {
                let master_id = *state.master_id.lock().unwrap();
                elevator.id == master_id
            } else {
                false
            }
        };

        // If the offline elevator was the master, choose a new master.
        if was_master {
            let new_master_id_opt = {
                let mut known_elevators = state.known_elevators.lock().unwrap();
                // Filter the alive elevators.
                let mut alive_elevators: Vec<&mut Cab> = known_elevators
                    .iter_mut()
                    .filter(|e| e.alive)
                    .collect();
                // Sort by ID so the one with the lowest ID comes first.
                alive_elevators.sort_by_key(|e| e.id);
                alive_elevators.get(0).map(|e| e.id)
            };


            if let Some(new_master_id) = new_master_id_opt {
                // Update the master id in the shared state.
                {
                    let mut master_id_lock = state.master_id.lock().unwrap();
                    *master_id_lock = new_master_id;
                }
                println!("Master offline, set new master to ID: {}", new_master_id);
                // Optionally, broadcast the new master status here.
            } else {
                println!("No alive elevators found to set as new master.");
            }
        } 
        else 
        {
            println!("Elevator ID:{} not found in known elevators.", cab.id);
        }

        // If this elevator is the new master, reassign orders.
        {
            let master_id = *state.master_id.lock().unwrap();
            if state.me_id == master_id {
                let orders_to_reassign = {
                    let known_elevators = state.known_elevators.lock().unwrap();
                    // Find the offline elevator and collect its hall orders.
                    if let Some(elevator) = known_elevators.iter().find(|e| e.id == cab.id) {
                        elevator
                            .queue
                            .iter()
                            .filter(|order| order.order_type == HALL_UP || order.order_type == HALL_DOWN)
                            .cloned()
                            .collect::<Vec<Order>>()
                    } else {
                        Vec::new()
                    }
                };
                if !orders_to_reassign.is_empty() {
                    println!("I am master, reassigning orders: {:?}", orders_to_reassign);
                    reassign_orders(&orders_to_reassign, &state, udp_handler, order_update_tx);
                } else {
                    println!("No hall orders found for reassignment.");
                }
            }
        }

        // Finally, remove all orders except cab orders from the offline elevator.
        {
            let mut known_elevators = state.known_elevators.lock().unwrap();
            if let Some(elevator) = known_elevators.iter_mut().find(|e| e.id == cab.id) {
                elevator.queue.retain(|order| order.order_type == CAB);
            }
        }
    }
}

/// handle_error_offline
/// # Arguments:
/// 
/// * `msg` - &UdpMsg - refrence to the UDP message that was recivecd.
/// * `known_elevators` - &mut Arc<Mutex<Vec<Cab>>> - List of active elevators.
/// * `&mut failed_orders` - &mut Arc<Mutex<Vec<Order>>> - list of orders that couldnt be distributed.
/// 
/// # Returns:
///
/// Returns - None - .
///
pub fn handle_remove_order(msg: &UdpMsg, state: Arc<SystemState>, light_update_tx: cbc::Sender<Vec<Order>>) {

    let elevator_from_msg = if let UdpData::Cab(cab) = &msg.data {
        cab
    } else {
        println!("ERROR: Expected UdpData::Cab but got something else");
        return;
    };
    
    //Find ID of elevator
    
    let remove_id = elevator_from_msg.id;

    println!("Removing order from ID: {}", remove_id);

    //Lock active elevators
    let mut known_elevators_locked = state.known_elevators.lock().unwrap();

    //Check for correct elevator in active elevators
    if let Some(elevator) = known_elevators_locked.iter_mut().find(|e| e.id == remove_id) {

        for order in &elevator_from_msg.queue {
    
            if let Some(index) = elevator.queue.iter().position(|o| o == order) {
                            
                elevator.queue.remove(index);
                println!("Order {:?} removed from elevator ID: {}", order, elevator.id);
                if elevator.id == state.me_id{
                    light_update_tx.send(elevator.queue.clone()).unwrap();
                }
                    
            } else {
                println!("ERROR: Elevator ID:{} does not have order {:?}", elevator.id, order); 
            }
                            
        }

    } else {
        println!("ERROR: No elevator data found in the message.");
    }

}


pub fn handle_im_alive(msg: &UdpMsg, state: Arc<SystemState>){
    //Extract updated cab data from message
    let updated_cab = if let UdpData::Cab(cab) = &msg.data{
        cab.clone()
    }else{
        println!("Couldnt read ImAlive message");
        return;
    };

    //Replace the old cab struct with the updated cab struct
    let mut known_elevators_locked = state.known_elevators.lock().unwrap();
    if let Some(sender_elevator) = known_elevators_locked.iter_mut().find(|e| e.id == msg.header.sender_id){
        println!("Updating alive elevator");
        sender_elevator.alive=true;
        sender_elevator.merge_with(&updated_cab);   //------------------------------------------------------------------------------PROBLEM?
        sender_elevator.last_lifesign = SystemTime::now();
        //Update last lifesign of that elevator

    } else {
        //Send a NewOnline message with that cab // ----------------------------------------------------------------------------------This will be corrected in next worldview as there will be a discrepancy
        println!("Elevator not known, running handle_new_online");
        drop(known_elevators_locked);
        handle_new_online(&msg, state);
    }
    
    
}

//Handle Order completed

pub fn handle_order_completed(msg: &UdpMsg, state: Arc<SystemState>, light_update_tx_clone: cbc::Sender<Vec<Order>>){

    //Extract updated cab data from message
    let completed_cab = if let UdpData::Cab(cab) = &msg.data{
        cab.clone()
    }else{
        println!("Couldnt read OrderComplete message");
        return;
    };

    let completed_order = match completed_cab.queue.first(){
        Some(order) => order.clone(),

        None => {
            println!("Completed order message contains no order");
            return;
        }
    };
 
    //Remove it from elevator orders
    let mut known_elevators_locked = state.known_elevators.lock().unwrap();
    if let Some(elevator) = known_elevators_locked.iter_mut().find(|e|e.id==completed_cab.id){
        if let Some(index) = elevator.queue.iter().position(|o|*o == completed_order){
            elevator.queue.remove(index);
            println!("Removed completed order {:?} from ID:{}",completed_order,elevator.id);
        }
    }
    
    let mut all_orders_locked = state.all_orders.lock().unwrap();
    if completed_order.order_type == CAB {
        if let Some(index) = all_orders_locked.iter().position(|order| (order.floor == completed_order.floor)&& (order.order_type == CAB)) {
            all_orders_locked.remove(index);
        }
    } else {
        if let Some(index) = all_orders_locked.iter().position(|order| 
            order.floor == completed_order.floor && order.order_type == completed_order.order_type) {
            all_orders_locked.remove(index);
        }
    }

            
}


//NEW_REQUEST
pub fn handle_new_request(msg: &UdpMsg, state: Arc<SystemState>,udp_handler: Arc<UdpHandler>, order_update_tx: cbc::Sender<Vec<Order>>,light_update_tx: cbc::Sender<Vec<Order>>){

    // Find order in message
    let new_order = if let UdpData::Order(order) = &msg.data{
        order.clone()
    }else{
        println!("Couldnt read NewRequest message");
        return;
    };

    println!("New request recived Floor:{}, Type{}",new_order.floor,new_order.order_type);

    //Lock list of all orders
    let mut all_orders_locked = state.all_orders.lock().unwrap(); 
    
    all_orders_locked.push(new_order.clone());
    drop(all_orders_locked);

    //Check if this elevator is master 
    let master_id_clone = state.master_id.lock().unwrap().clone();
    let is_master = state.me_id == master_id_clone;


    //IF New Request is CAB order
    if new_order.order_type == CAB{
        // Lock the known elevators and find the elevator that matches the sender id.
        let mut known_elevators_locked = state.known_elevators.lock().unwrap();
        if let Some(sender_elevator) = known_elevators_locked.iter_mut().find(|e| e.id == msg.header.sender_id) {
            sender_elevator.queue.push(new_order.clone());
            if sender_elevator.id == state.me_id{
                light_update_tx.send(sender_elevator.queue.clone()).unwrap();
            }
            println!("Entered call type cab");
            if is_master {
                // Capture necessary data (elevator id) before dropping the lock.
                let elevator_id = sender_elevator.id;
                // Lock is dropped here when the block ends.
                drop(known_elevators_locked);
                give_order(elevator_id, vec![&new_order], &state, &udp_handler);
                println!("Added CAB order to elevator ID: {}", elevator_id);
            }
        }
        else{
            println!("Elevator with NewRequest CAB is not active ID:{}", msg.header.sender_id)
        }
        order_update_tx.send(vec![new_order.clone()]).unwrap();    
    
    }else {
        
        if is_master{
            let known_elevators_locked = state.known_elevators.lock().unwrap();
            let alive_elevators: Vec<Cab> = known_elevators_locked.iter().filter(|e| e.alive).cloned().collect();

            if !alive_elevators.is_empty() {
                let best_elevators = best_to_worst_elevator(&new_order, &alive_elevators);
                drop(known_elevators_locked);

                let best_elevator = match best_elevators.first() {
                    Some(elevator) => {
                        println!("Assigning new hallcall to {:?}", elevator);
                        elevator
                    }
                    None => {
                        println!("No available elevator to assign the order, assigning to self");
                        &state.me_id
                    }
                };

                let success = give_order(*best_elevator, vec![&new_order], &state, &udp_handler);
                
                if !success{
                    let mut known_elevators_locked = state.known_elevators.lock().unwrap();
                    known_elevators_locked.get_mut(0).unwrap().queue.push(new_order.clone());
                }

                order_update_tx.send(vec![new_order.clone()]).unwrap();
            } 
        }       
    }
    println!("THIS ORDER UPDATE 2 of 4");
    order_update_tx.send(vec![new_order.clone()]).unwrap();
}