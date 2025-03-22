//! ## UDP Module
//! This module provides structs and functions for sending and receiving UDP messages
//!
//! ## The structs includes:
//! - **UdpMsg**: Contains the data that is being sendt aswell as a header that describes the message
//! - **UdpHeader**: Contains information about the message such as sender, checksum and message type
//! 
//! ## Message Types:
//! - **Wordview:** Synchronizes the system's state across the diffrent nodes, only sent by master node.
//! - **Ack / Nak:** Acknowledgment and negative acknowledgment for confirmation of recvied messages.
//! - **New_Order:** Represents a new floor request for a specific elevator.
//! - **New_Master:** Informs the system that there ahs been a change of master.
//! - **New_Online:** Informs that a new elevator has joined the system/gone online.
//! - **Error_Worldview:** Reports inconsistencies in worldview synchronization.
//! - **Error_Offline:** Handles elevator disconnections.
//! 
//! ## The functions includes:
//! - 'make_udp_msg'  Formats a UDP message.
//! - 'udp_receive'   Categorizes the recived message and handels accordingly.
//! - 'handle_"Message_type"' handels each spesific mesesage type.
//! - 'serialize'     serializes UDP messages for transmission.           
//! - 'deserialize'    deserializes transmitted udp messages.
//! - 'calc_checksum'  calculates checksum to ensure message integrity.
//! - 'comp_checksum'  compares checksum of recived message to the calculated checksum.
//! - 'udp_send'       sending of udp messages without requirement for acknowledment.
//! - 'udp_broadcast'  broadcasts UDP messages.
//! - 'udp_recive_ensure'  recives UDP messages and responds with ACK if message is accepted/correct.
//! - 'udp_send_ensure'  sends UDP messages and waits for ACK,if not recvied within timeout, it resends untill it runs out of retries.
//!
//! ## Dependencies
//! **The following dependencies have to be included in `Cargo.toml`:**
//! 
//! ```toml
//! [dependencies]
//! serde = { version = "1", features = ["derive"] }
//! bincode = "1"
//! sha2 = { version = "0.11.0-pre.4" }
//! ```
//! these are primarily used for serialization/deserialization and calculation hash for checksum. 

#[allow(unused_imports)]
#[allow(unused_variables)]
#[allow(non_camel_case_types)]

//----------------------------------------------Imports
use std::net::{SocketAddr, UdpSocket}; // https://doc.rust-lang.org/std/net/struct.UdpSocket.html
                                       //use std::sync::{Arc, Mutex};          // https://doc.rust-lang.org/std/sync/struct.Mutex.html
use serde::{Deserialize, Serialize}; // https://serde.rs/impl-serialize.html         //Add to Cargo.toml file, Check comment above
                                     // https://docs.rs/serde/latest/serde/ser/trait.Serialize.html#tymethod.serialize
use bincode; 
// https://docs.rs/bincode/latest/bincode/      //Add to Cargo.toml file, Check comment above
use sha2::{Digest, Sha256}; // https://docs.rs/sha2/latest/sha2/            //Add to Cargo.toml file, Check comment above
use std::time::{Duration,Instant}; // https://doc.rust-lang.org/std/time/struct.Duration.html
// use std::thread::sleep; // https://doc.rust-lang.org/std/thread/fn.sleep.html
use std::sync::{Mutex,Arc};

use crate::modules::order_object::order_init::Order;
use crate::modules::elevator_object::elevator_init::SystemState;
use crate::modules::cab::cab::Cab;
use crate::modules::master::master::{handle_multiple_masters,Role,correct_master_worldview,generate_worldview, reassign_orders};
use crate::modules::slave::slave::update_from_worldview;

pub use crate::modules::elevator_object::*;
pub use elevator_init::Elevator;
pub use alias_lib::{HALL_DOWN, HALL_UP,CAB, DIRN_DOWN, DIRN_UP, DIRN_STOP};


//----------------------------------------------Enum
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub enum MessageType {

    Worldview,
    Ack,
    Nak,
    NewOrder,
    NewMaster,
    NewOnline,
    RequestQueue,
    RespondQueue,
    ErrorWorldview,
    ErrorOffline,
    RequestResend,
    OrderComplete,
    RemoveOrder,
    NewRequest,
}

//----------------------------------------------Structs
#[derive(Debug, Serialize, PartialEq, Deserialize, Clone)] // this is needed to serialize message
//UDP Header
pub struct UdpHeader {
    pub sender_id: u8,             // ID of the sender of the message.
    pub message_type: MessageType, // ID for what kind of message it is, e.g. Button press, or Update queue.
    pub checksum: Vec<u8>,         // Hash of data to check message integrity.
}

#[derive(Debug, Serialize, PartialEq, Deserialize, Clone)] // this is needed to serialize message
//UDP Message Struct
pub struct UdpMsg {
    pub header: UdpHeader,       // Header struct containing information about the message itself
    pub data: UdpData,        // Data so be sent.
}


pub struct UdpHandler {
    sender_socket: UdpSocket,
    receiver_socket: UdpSocket,
}

#[derive(Debug, Serialize, PartialEq, Deserialize, Clone)]
pub enum UdpData {
    Cabs(Vec<Cab>),
    Cab(Cab),
    Orders(Vec<Order>),
    Order(Order),
    None,
}


//----------------------------------------------Functions


impl UdpHandler {

    /// Sends a UDP message
    pub fn send(&self, target_address: &SocketAddr, msg: &UdpMsg) -> bool {
        let data = serialize(msg);
        match self.sender_socket.send_to(&data, target_address) {
            Ok(_) => {
                println!("Message sent to: {}", target_address);
                return true;
            }
            Err(e) => {
                eprintln!("Error sending message: {}", e);
                return false;
            }
        }
    }


    /// receive
    /// 
    /// # Arguments:
    /// 
    /// * `max_wait` - u32 - maximum wait time in milliscounds.
    /// * `state` - &mut SystemState  - mutable refrence to systemstate.
    /// 
    /// # Returns:
    ///
    /// Returns -Option(UdpMsg)- Handels message based on message type and returns either a message or none depending on message.
    ///
    pub fn receive(&self, max_wait: u32, state:&mut SystemState) -> Option<UdpMsg> {

        //Set socket from udp.handler
        self.receiver_socket
            .set_read_timeout(Some(Duration::from_millis(max_wait as u64)))
            .expect(&format!("Failed to set read timeout of {} ms", max_wait));

        let mut buffer = [0; 1024];

        match self.receiver_socket.recv_from(&mut buffer) {
            Ok((size, sender)) => {
                println!("Received message of size {} from {}", size, sender);

                if let Some(msg) = deserialize(&buffer[..size]) {
                    println!("Message type: {:?}", msg.header.message_type);

                    match msg.header.message_type{
                        MessageType::Worldview => {handle_worldview(state, &msg);},
                        MessageType::Ack => {handle_ack(&msg, &mut state.sent_messages);},
                        MessageType::Nak => {handle_nak(&msg, &mut state.sent_messages, &sender, &self);},
                        MessageType::NewOrder => {handle_new_order(&msg, &sender, state, &self);},
                        MessageType::NewMaster => {handle_new_master(&msg, &state.active_elevators);},
                        MessageType::NewOnline => {handle_new_online(&msg, state);},
                        MessageType::ErrorWorldview => {handle_error_worldview(&msg, &state.active_elevators);},
                        MessageType::ErrorOffline => {handle_error_offline(&msg, state, &self);},
                        MessageType::OrderComplete => {handle_remove_order(&msg, &mut state.active_elevators);},
                        MessageType::NewRequest => {handle_new_request(&msg, &sender, state, &self);},
                        _ => println!("Unreadable message received from {}", sender),
                    }
                        return Some(msg);
                } else {
                    println!("Failed to deserialize message from {}", sender);
                    return None;
                }
            }
            Err(e) => {
                println!("Failed to receive message: {}", e);
                return None;
            }
        }
    }
}

pub fn handle_new_request(msg: &UdpMsg, sender_address: &SocketAddr, state: &mut SystemState,udp_handler: &UdpHandler){
    //Lock active elevators
    let mut active_elevators_locked = state.active_elevators.lock().unwrap(); 
    //Find elevator with mathcing ID and update queue
    if let Some(sender) = active_elevators_locked.iter_mut().find(|elevator| elevator.id == msg.header.sender_id) {
        let order = &sender.queue.first().unwrap();
        if (*(*order)).order_type == HALL_DOWN||(*(*order)).order_type == HALL_UP{
            sender.queue.remove(1);
        };
            

    }        
            
}

/// Creates a new UDP handler with a bound sockets based on this elevator
pub fn init_udp_handler(me: Cab) -> UdpHandler {

    let sender_socket = UdpSocket::bind(me.out_address).expect("Could not bind UDP socket");
    let receiver_socket = UdpSocket::bind(me.inn_address).expect("Could not bind UDP receiver socket");
    sender_socket.set_nonblocking(true).expect("Failed to set non-blocking mode");
    receiver_socket.set_nonblocking(true).expect("Failed to set non-blocking mode");
    return UdpHandler{sender_socket,receiver_socket};
}

///make_udp_msg
/// 
/// # Arguments:
/// 
/// * `message_type` - MessageType - what kind of message, check enum MessageType
/// * `message` - Vec<Cab> The message to be sendt
/// # Returns:
///
/// Returns -UdpMsg- The message that has been generated.
///
pub fn make_udp_msg(sender_id: u8,message_type: MessageType, message: UdpData) -> UdpMsg {
    let hash = calc_checksum(&message);
    let overhead = UdpHeader {
        sender_id: sender_id,
        message_type: message_type,
        checksum: hash,
    };

    let msg = UdpMsg {
        header: overhead,
        data: message.clone(),
    };
    return msg;
}




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
pub fn handle_worldview(state: &mut SystemState, msg: &UdpMsg) {
    println!("Updating worldview...");

    //Update last lifesign and last worldview
    let mut last_lifesign_locked = state.last_lifesign.lock().unwrap();
    *last_lifesign_locked = Instant::now();
    drop(last_lifesign_locked);

    let mut new_worldview = state.last_worldview.lock().unwrap();
    *new_worldview = msg.clone();
    drop(new_worldview);
    
    let elevators = if let UdpData::Cabs(elevator) = &msg.data{
        elevator
    }
    else{
        println!("Wrong data in message for worldview");
        return;
    };

    update_from_worldview(state, &elevators);
    let active_elevators: Vec<Cab> = {
    let active_elevators_locked = state.active_elevators.lock().unwrap();
    active_elevators_locked.clone() 
    };
     

    generate_worldview(&active_elevators);
    handle_multiple_masters(state, &msg.header.sender_id);
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
pub fn handle_ack(msg: &UdpMsg, sent_messages: &mut Arc<Mutex<Vec<UdpMsg>>>) {
    println!("Received ACK from ID: {}", msg.header.sender_id);

    let mut sent_messages_locked = sent_messages.lock().unwrap();

    // Check if this ACK matches sent message

    if let Some(index) = sent_messages_locked.iter().position(|m| calc_checksum(&msg.data) == msg.header.checksum) {
        println!("ACK matches message with checksum: {:?}", msg.data);
                
        // Remove acknowledged message from tracking
        sent_messages_locked.remove(index);
    } else { 
        println!("ERROR: Received ACK with unknown checksum {:?}", msg.data);
    }

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
pub fn handle_nak(msg: &UdpMsg, sent_messages: &Arc<Mutex<Vec<UdpMsg>>>, target_address: &SocketAddr,udp_handler: &UdpHandler) {
    println!("Received NAK from ID: {}", msg.header.sender_id);

    // Check if this NAK matches sent message
    let sent_messages_locked = sent_messages.lock().unwrap();
    if let Some(index) = sent_messages_locked.iter().position(|m| calc_checksum(&m.data) == msg.header.checksum) {
        println!("NAK matches message with checksum: {:?}. Resending...", msg.header.checksum);
        // Resend the message                udp_handler.send(target_address, &sent_messages_locked[index]);
    } else {
        println!("ERROR: Received NAK with unknown checksum {:?}", msg.header.checksum);
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

pub fn handle_new_order(msg: &UdpMsg, sender_address: &SocketAddr, state: &mut SystemState,udp_handler: &UdpHandler) -> bool {
    println!("New order received ID: {}", msg.header.sender_id);

    let elevator = if let UdpData::Cab(cab) = &msg.data {
        cab
    } else {
        println!("ERROR: Wrong UdpData type for NewOrder");
        return false;
    };

    let elevator_id = elevator.id;

    //Lock active elevators
    let mut active_elevators_locked = state.active_elevators.lock().unwrap(); 

    //Find elevator with mathcing ID and update queue
    if let Some(update_elevator) = active_elevators_locked.iter_mut().find(|e| e.id == elevator_id){
        for order in &elevator.queue {
            if !update_elevator.queue.contains(&order){
                update_elevator.queue.push(order.clone());
                println!("Order {:?} successfully added to elevator {}.", order, elevator.id);
            }else {
                println!("Order {:?} already in queue for elevator {}.", order, elevator.id);
            }
        }
    }
    //Send Ack to sender
    return udp_ack(*sender_address, &msg, elevator.id, udp_handler);
}

/// handle_new_master
/// # Arguments:
/// 
/// * `msg` - UdpMsg - recived message.
/// * `active_elevators` - &Arc<Mutex<Vec<Cab>>> - Vector of active elevators.
/// 
/// # Returns:
///
/// Returns -None- .
///
pub fn handle_new_master(msg: &UdpMsg, active_elevators: &Arc<Mutex<Vec<Cab>>>) {
    println!("New master detected, ID: {}", msg.header.sender_id);

    

    // Set current master's role to Slave
    let mut active_elevators_locked = active_elevators.lock().unwrap();
    if let Some(current_master) = active_elevators_locked.iter_mut().find(|elevator| elevator.role == Role::Master) {
        println!("Changing current master (ID: {}) to slave.", current_master.id);
        current_master.role = Role::Slave;
    } else {
        println!("ERROR: No active master found.");
    }

    // Set new master
    if let Some(new_master) = active_elevators_locked.iter_mut().find(|elevator| elevator.id == msg.header.sender_id) {
        println!("Updating elevator ID {} to Master.", msg.header.sender_id);
        new_master.role = Role::Master;
    } else {
        println!("Error: Elevator ID {} not found in active list.", msg.header.sender_id);
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
pub fn handle_new_online(msg: &UdpMsg, state: &mut SystemState) -> bool {
    println!("New elevator online, ID: {}", msg.header.sender_id);

    //Lock active elevaotrs
    let active_elevators_locked = state.active_elevators.lock().unwrap();

    // Check if elevator is already active
    if active_elevators_locked.iter().any(|e| e.id == msg.header.sender_id) {
        println!("Elevator ID:{} is already active.", msg.header.sender_id);
        return true;
    }
    //Release active elevators
    drop(active_elevators_locked); 

    let msg_elevator = if let UdpData::Cab(cab) = &msg.data {
        cab
    } else {
        println!("Error: Wrong UdpData for message type");
        return false;
    };

    // Create new elevator
    let new_elevator = Cab {
        inn_address: msg_elevator.inn_address,
        out_address: msg_elevator.out_address,
        num_floors: msg_elevator.num_floors,
        id: msg_elevator.id,
        current_floor: msg_elevator.current_floor,
        queue: msg_elevator.queue.clone(),
        status: msg_elevator.status.clone(),
        direction: msg_elevator.direction.clone(),
        role: msg_elevator.role.clone(),
    };

    // Lock again and add the new elevator
    let mut active_elevators_locked = state.active_elevators.lock().unwrap();
    active_elevators_locked.push(new_elevator);
    drop(active_elevators_locked); 

    println!("Added new elevator ID {}.", msg.header.sender_id);
    return true;
}


/// handle_error_worldview
/// # Arguments:
/// 
/// * `msg` - &UdpMsg - refrence to UDP message.
/// * `active_elevators` - &Arc<Mutex<Vec<Cab>>> - List of active elevators .
/// 
/// # Returns:
///
/// Returns -None- .
///
pub fn handle_error_worldview(msg: &UdpMsg, active_elevators: &Arc<Mutex<Vec<Cab>>>) {
    println!("EROR: Worldview error reported by ID: {}", msg.header.sender_id);

    // List of orders from sender
    let mut missing_orders = if let UdpData::Cabs(cabs) = &msg.data {
        cabs.clone()
    } else {
        println!("ERROR: Expected UdpData::Cabs but got something else");
        return;
    };

    // Compare and correct worldview based on received data
    if correct_master_worldview(&mut missing_orders, active_elevators) {
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
pub fn handle_error_offline(msg: &UdpMsg,state: &mut SystemState ,udp_handler: &UdpHandler) {
    println!("Elevator {} went offline. Reassigning orders", msg.header.sender_id);

    let mut removed_elevator: Option<Cab> = None;
    let mut active_elevators_locked = state.active_elevators.lock().unwrap();

    // Check if active elevators contains, retain keeps only elements that return true
    active_elevators_locked.retain(|e| {
        if e.id == msg.header.sender_id {
            removed_elevator = Some(e.clone());
            //Remove this elevator
            return false;  
        } else {
            //Keep this elevator
            return true; 
        }
    });

    //Release active elevators
    drop(active_elevators_locked);

    // Check if the elevator was removed
    if let Some(offline_elevator) = removed_elevator {
        println!("Removed elevator ID: {} from active list.", msg.header.sender_id);

        // Extract orders from the offline elevator
        let orders = offline_elevator.queue.clone();
        println!("Reassigning orders, if any: {:?}", orders);
        let order_ids: Vec<Order> = orders.iter().map(|order| (*order).clone()).collect();
        reassign_orders(&order_ids, state ,udp_handler);
    } else {
        println!("ERROR: Elevator ID {} was not found in active list.", msg.header.sender_id);
    }
}

/// handle_error_offline
/// # Arguments:
/// 
/// * `msg` - &UdpMsg - refrence to the UDP message that was recivecd.
/// * `active_elevators` - &mut Arc<Mutex<Vec<Cab>>> - List of active elevators.
/// * `&mut failed_orders` - &mut Arc<Mutex<Vec<Order>>> - list of orders that couldnt be distributed.
/// 
/// # Returns:
///
/// Returns - None - .
///
pub fn handle_remove_order(msg: &UdpMsg, active_elevators: &mut Arc<Mutex<Vec<Cab>>>) {

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
    let mut active_elevators_locked = active_elevators.lock().unwrap();

    //Check for correct elevator in active elevators
    if let Some(elevator) = active_elevators_locked.iter_mut().find(|e| e.id == remove_id) {
    
        if let Some(order) = elevator_from_msg.queue.first() {
                    
            if let Some(index) = elevator.queue.iter().position(|o| o == order) {
                elevator.queue.remove(index);
                println!("Order {:?} removed from elevator ID: {}", order, elevator.id);
            } else {
                println!("ERROR: Elevator ID:{} does not have order {:?}", elevator.id, order); 
            }               
                    
        } else {
                println!("ERROR: No orders found in the received elevator.");
        }

    } else {
        println!("ERROR: No elevator data found in the message.");
    }
}



/// serialize
/// Split UdpMsg into bytes for easier transmission
/// 
/// # Arguments:
/// 
/// * `msg` - &UdpMsg - refrence to message
/// 
/// # Returns:
///
/// Returns - Vec<u8>- a string of the serialized message.
///
pub fn serialize(msg: &UdpMsg) -> Vec<u8> {
    let serialized_msg = bincode::serialize(msg).expect("Failed to serialize message");
    return serialized_msg;
}

/// deserialize
/// Combine bytes in message buffer into UdpMsg
///
/// # Arguments:
/// 
/// * `buffer` - &[u8] - refrence to the buffer containing the serialized message.
/// 
/// # Returns:
///
/// Returns - Option<UdpMsg>- .returns either the deserialized message or none
///
pub fn deserialize(buffer: &[u8]) -> Option<UdpMsg> {
    match bincode::deserialize::<UdpMsg>(buffer) {
        Ok(msg) => {
            if data_valid_for_type(&msg) {
                Some(msg)
            } else {
                println!("Invalid data for message type");
                None
            }
        }
        Err(e) => {
            println!("Failed to deserialize message: {}", e);
            None
        }
    }
}

/// data_valid_for_type
/// Checks that the message contains correct data structure for message type to ensure correct deserialization.
/// used primarily in derserialization()
/// 
/// # Arguments:
/// 
/// * `msg` - &UdpMsg - refrence to UDP message 
/// 
/// #Returns
/// 
/// Returns - bool - returns true if order har correct Data type according to MessageType
/// 
fn data_valid_for_type(msg: &UdpMsg) -> bool {
    match (&msg.header.message_type, &msg.data) {
        (MessageType::NewOrder, UdpData::Cabs(_)) => true,
        (MessageType::Worldview, UdpData::Cabs(_)) => true,
        (MessageType::OrderComplete, UdpData::Cab(_)) => true,
        (MessageType::NewRequest, UdpData::Order(_)) => true,
        (MessageType::ErrorWorldview, UdpData::Cabs(_)) => true,
        (MessageType::ErrorOffline, UdpData::Cab(_)) => true,
        (MessageType::NewMaster, UdpData::Cab(_)) => true,
        (MessageType::NewOnline, UdpData::Cab(_)) => true,
        (MessageType::Ack, UdpData::None) => true,
        (MessageType::Nak, UdpData::None) => true,
        _ => false,
    }
}

/// calc_checksum
/// Calculate Checksum.
/// 
/// # Arguments:
/// 
/// * `data` - &Vec<elevator> - refrence to list of elevators.
/// 
/// # Returns:
///
/// Returns - Vec<u8>- returns the hashed string .
///
pub fn calc_checksum(data: &UdpData) -> Vec<u8> {
    let serialized_data = bincode::serialize(data).expect("Failed to serialize data");
    let mut hasher = Sha256::new();
    Digest::update(&mut hasher, &serialized_data);
    let hash = hasher.finalize();
    return hash.as_slice().to_vec();
}

// comp_checksum
/// Compare checksums of a message's data, with its attached checksum.
/// 
/// # Arguments:
/// 
/// * `msg` - &UdpMsg - refrence to message.
/// 
/// # Returns:
///
/// Returns - bool - returns 'true' if they match or 'false' if they dont.
///
pub fn comp_checksum(msg: &UdpMsg) -> bool {
    return calc_checksum(&msg.data) == msg.header.checksum;
}


//-------------------------MOVE TO HANDLER STRUCT
///udp_ack
///ACK, Responds to original messag with ACK, checksum of original message is used as data to ensure which message it is responding to.
/// 
/// # Arguments:
/// 
/// * `socket` - &UdpSocket - 
/// * `target_adress` - SocketAddr - .
/// * `original_msg` - &UdpMsg - .
/// * `sender_id` - u8 - .
/// * `udp_handler` - &UdpHandler -
/// 
/// # Returns:
///
/// Returns - bool - returns 'true' if succsessful and 'false' if not.
///
pub fn udp_ack(target_address: SocketAddr, original_msg: &UdpMsg, sender_id: u8,udp_handler: &UdpHandler) -> bool {
    let checksum = calc_checksum(&original_msg.data); // Compute checksum of original data

    let ack_msg = UdpMsg {
        header: UdpHeader {
            sender_id,
            message_type: MessageType::Ack, 
            checksum: checksum.clone(),   
        },
        data: UdpData::None, 
    };

    return udp_handler.send(&target_address, &ack_msg);
}

//------------------------MOVE TO HANDLER STRUCT
/// udp_nak
///NAK, Responds to original messag with NAK, checksum of original message is used as data to ensure which message it is responding to.
/// 
/// # Arguments:
/// 
/// * `target_adress` - SocketAddr - .
/// * `original_msg` - &UdpMsg - .
/// * `sender_id` - u8 - .
/// * `udp_handler` -&UdpHandler- 
///  
/// # Returns:
///
/// Returns - bool - returns 'true' if succsessful and 'false' if not.
///
pub fn udp_nak(target_address: SocketAddr, original_msg: &UdpMsg, sender_id: u8,udp_handler: &UdpHandler) -> bool {
    let checksum = calc_checksum(&original_msg.data); // Compute checksum of original data

    let nak_msg = UdpMsg {
        header: UdpHeader {
            sender_id,
            message_type: MessageType::Nak, 
            checksum: checksum.clone(),  
        },
        data: UdpData::None, 
    };

    return udp_handler.send(&target_address, &nak_msg);
}


///udp_broadcast
///Broadcast
/// 
/// # Arguments:
/// 
/// * `msg` - &UdpMsg - .
/// 
/// # Returns:
///
/// Returns - None - .
///
pub fn udp_broadcast(msg: &UdpMsg) -> bool {
    let socket = UdpSocket::bind("0.0.0.0:0").expect("Failed to bind socket for broadcast");
    socket
        .set_broadcast(true)
        .expect("failed to activate broadcast");

    let msg = serialize(msg);
    let target_address = "255.255.255.255;20000";

    match socket.send_to(&msg, target_address) {
        Ok(_) => {
            println!("Broadcast successful");
            return true;
        }
        Err(e) => {
            eprintln!("Error sending data: {}", e);
            
        }
    }
    return false;
}   


/* 
///udp_send_ensure
/// Sending UDP, with retry
/// 
/// # Arguments:
/// 
/// * `socket` - &UdpSocket - .
/// * `target_addr` - &str - .
/// * `msg` - &UdpMsg - .
/// * `max_retry` -  - .
/// * `sent_messages` - &mut Vec<UdpMsg> - ..
/// 
/// # Returns:
///
/// Returns - bool - returns 'true' else returns 'false'.
///
pub fn udp_send_ensure(socket: &UdpSocket, target_addr: &str, msg: &UdpMsg, max_retry: u8, sent_messages: &mut Vec<UdpMsg>) -> bool {
    let mut retries = max_retry;
    let msg_checksum = calc_checksum(&msg.data);

    // Store message in tracking list
    sent_messages.push(msg.clone());

    while retries > 0 {

        // Send the message
        if udp_send(socket, target_addr.parse().unwrap(), msg) {
            println!("Sent message to {}", target_addr);
        } else {
            println!("Error sending message, retrying...");
        }

        // Wait for a response
        let mut buffer = [0; 1024];
        match socket.recv_from(&mut buffer) {
            Ok((size, rec_addr)) => {
                if rec_addr == target_addr.parse().unwrap(){
                    if let Some(response_msg) = deserialize(&buffer[..size]) {
                        match response_msg.header.message_type {
                            MessageType::Ack => {
                                if response_msg.header.checksum == msg_checksum {
                                    println!("ACK received for message");
                                    sent_messages.retain(|m| calc_checksum(&m.data) != msg_checksum);
                                    return true; 
                                } else {
                                    println!("ERROR: Received ACK wrong checksum!");
                                }
                            }
                            MessageType::Nak => {
                                if response_msg.header.checksum == msg_checksum {
                                    println!("NAK received, resending message...");
                                } else {
                                    println!("ERROR: Received NAK with unknown checksum!");
                                }
                            }
                            _ => {
                                println!("ERROR: Unexpected message type received");
                            }
                        }
                    }
                }else{
                    println!("Couldnt read message");
                }
            }
            Err(e) => {
                println!("No response received before timeout, retrying... [{} retries left]", retries);
            }
        }

        retries -= 1;
    }

    println!("Failed to send message after {} retries.", max_retry);
    false
}

*/

/* 
/// udp_receive_ensure
/// Reciving UDP, with ACK
/// 
/// # Arguments:
/// 
/// * `socket: &UdpSocket` -  -
/// * `max_wait` - u8 - .
/// * `receiver_id` - u8 - .
/// 
/// # Returns:
///
/// Returns -Option<UdpMsg> - returns either a UDP Message or nothing .
///
pub fn udp_receive_ensure(socket: &UdpSocket, max_wait: u8, receiver_id: u8) -> Option<UdpMsg> {
    socket
        .set_read_timeout(Some(Duration::from_secs(max_wait.into())))
        .expect("Failed to set read timeout");

    let mut buffer = [0; 1024];

    match socket.recv_from(&mut buffer) {
        Ok((size, sender_addr)) => {
            if let Some(msg) = deserialize(&buffer[..size]) {
                let received_checksum = calc_checksum(&msg.data);

                if received_checksum == msg.header.checksum {
                    println!("Valid message received from {} with checksum {:?}", msg.header.sender_id, received_checksum);

                    udp_ack(socket, sender_addr, &msg, receiver_id);

                    return Some(msg);
                } else {
                    println!("Checksum does not match. Expected {:?}, got {:?}. Sending NAK...", msg.header.checksum, received_checksum);

                    udp_nak(socket, sender_addr, &msg, receiver_id);
                }
            } else {
                println!("ERROR: Failed to deserialize message from {}", sender_addr);
            }
        }
        Err(e) => {
            eprintln!("Error receiving message: {}", e);
        }
    }

    return None;
}
*/

