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
//! - 'make_Udp_msg'  Formats a UDP message.
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



//----------------------------------------------Imports
use std::net::{SocketAddr, UdpSocket}; // https://doc.rust-lang.org/std/net/struct.UdpSocket.html
                                       //use std::sync::{Arc, Mutex};          // https://doc.rust-lang.org/std/sync/struct.Mutex.html
use serde::{Deserialize, Serialize}; // https://serde.rs/impl-serialize.html         //Add to Cargo.toml file, Check comment above
                                     // https://docs.rs/serde/latest/serde/ser/trait.Serialize.html#tymethod.serialize
use bincode; use sha2::digest::Update;
// https://docs.rs/bincode/latest/bincode/      //Add to Cargo.toml file, Check comment above
use sha2::{Digest, Sha256}; // https://docs.rs/sha2/latest/sha2/            //Add to Cargo.toml file, Check comment above
use std::time::Duration; // https://doc.rust-lang.org/std/time/struct.Duration.html
use std::thread::sleep; // https://doc.rust-lang.org/std/thread/fn.sleep.html


use crate::modules::order_object::order_init::Order;
use crate::modules::elevator_object::elevator_init::Elevator;
use crate::modules::slave;
use crate::modules::master::{Worldview,handle_multiple_masters,Role,reassign_orders,correct_master_worldview};
use crate::modules::order_object::order_init::Order;


static mut failed_orders: Vec<Order> = Vec::new(); //MAKE THIS GLOBAL


//----------------------------------------------Enum
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum MessageType {

    Wordview,
    Ack,
    Nak,
    New_Order,
    New_Master,
    New_Online,
    Request_Queue,
    Respond_Queue,
    Error_Worldview,
    Error_Offline,
    Request_Resend,
    Order_Complete,
}

//----------------------------------------------Structs
#[derive(Debug, Serialize, Deserialize, Clone)] // this is needed to serialize message
//UDP Header
pub struct UdpHeader {
    pub sender_id: u8,            // ID of the sender of the message.
    pub message_type: MessageType, // ID for what kind of message it is, e.g. Button press, or Update queue.
    pub checksum: Vec<u8>,        // Hash of data to check message integrity.
}

#[derive(Debug, Serialize, Deserialize, Clone)] // this is needed to serialize message
                                                //UDP Message Struct
pub struct UdpMsg {
    pub header: UdpHeader, // Header struct containing information about the message itself
    pub data: Vec<Elevator>,     // Data so be sent.
}

pub struct UdpHandler {
    sender_socket: UdpSocket,
    receiver_socket: UdpSocket,
}


//----------------------------------------------Functions


impl UdpHandler {

    /// Creates a new UDP handler with a bound sockets
    pub fn new(me: Elevator) -> Self {
        let sender_socket = UdpSocket::bind(me.out_address).expect("Could not bind UDP socket");
        sender_socket.set_nonblocking(true).expect("Failed to set non-blocking mode");
        let receiver_socket = UdpSocket::bind(me.inn_address).expect("Could not bind UDP receiver socket");
        receiver_socket.set_nonblocking(true).expect("Failed to set non-blocking mode");
        return UdpHandler{sender_socket,receiver_socket};
    }

    /// Sends a UDP message
    pub fn udp_send(&self, target_address: SocketAddr, msg: &UdpMsg) -> bool {
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


    /// udp_receive
    /// 
    /// # Arguments:
    /// 
    /// * `socket` - &UdpSocket - what socket we are reading from.
    /// * `max_wait` - u32 - maximum waittime in milliscounds.
    /// * `slave` - &Elevator - Refrence to elevator.
    /// * `me` - &Elevator  - Refrence to this elevator.
    /// * `worldview` - &mut Worldview - refrence to worldview.
    /// 
    /// # Returns:
    ///
    /// Returns -Option(UdpMsg)- Handels message based on message type and returns either a message or none depending on message.
    ///
    pub fn udp_receive(&self, max_wait: u32, slave: &mut Elevator, me: &Elevator, active_elevators: &mut <Elevator>) -> Option<UdpMsg> {
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
                        MessageType::Wordview => {handle_wordview(slave, me, active_elevators, msg);},
                        MessageType::Ack => {handle_ack(msg, &mut sent_messages);},
                        MessageType::Nak => {handle_nak(msg, &mut sent_messages, &self.sender_socket, sender);},
                        MessageType::New_Order => {handle_new_order(slave, msg, &self.sender_socket, &sender);},
                        MessageType::New_Master => {handle_new_master(msg, &mut active_elevators);;},
                        MessageType::New_Online => {handle_new_online(msg,&mut *active_elevators);},
                        MessageType::Error_Worldview => {handle_error_worldview(msg,&mut *active_elevators);},
                        MessageType::Error_Offline => {handle_error_offline(msg, &self.sender_socket, &mut *active_elevators);},
                        MessageType::Order_Complete => {handle_remove_order(msg, &mut *active_elevators, &mut failed_orders);},
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

///make_Udp_msg
/// 
/// # Arguments:
/// 
/// * `elevator` - crate::modules::elevator::Elevator - . Sender
/// * `message_type` - MessageType - what kind of message, check enum MessageType
/// * `message` - Vec<Elevator> The message to be sendt
/// 
/// 
/// # Returns:
///
/// Returns - - .
///
pub fn make_Udp_msg(elevator: crate::modules::elevator::Elevator, message_type: MessageType, message: Vec<Elevator>) -> UdpMsg {
    let hash = calc_checksum(&message);
    let mut overhead = UdpHeader {
        sender_id: elevator.ID,
        message_type: MessageType,
        checksum: hash,
    };

    let msg = UdpMsg {
        header: overhead,
        data: message,
    };
    return msg;
}




/// handle_worldview
/// # Arguments:
/// 
/// * `slave` - &Elevator - referance to elevator.
/// * `worldview` - &mut Worldview - current worldview.
/// * `msg` - UdpMsg - Recived message.
/// 
/// # Returns:
///
/// Returns - NONE - .
///
pub fn handle_wordview(slave: &mut Elevator, me: &Elevator, active_elevators, msg: UdpMsg) {
    println!("Updating worldview...");
    update_from_worldview(slave, msg.data.clone());
    generate_worldview(active_elevators);
    handle_multiple_masters(me, slave);
}

/// handle_ack
/// 
/// # Arguments:
/// 
/// * `msg` - UdpMsg - Recived message .
/// * `sent_messages` &mut Vec<UdpMsg>-  -.
/// 
/// # Returns:
///
/// Returns -NONE- .
///
pub fn handle_ack(msg: UdpMsg, sent_messages: &mut Vec<UdpMsg>) {
    println!("Received ACK from ID: {}", msg.header.sender_id);

    // Check if this ACK matches sent message
    if let Some(index) = sent_messages.iter().position(|m| calc_checksum(&m.data) == msg.header.checksum) {
        println!("ACK matches message with checksum: {:?}", msg.data);
        
        // Remove acknowledged message from tracking
        sent_messages.remove(index);
    } else {
        println!("ERROR: Received ACK with unknown checksum {:?}", msg.data);
    }
}

///handle_nack
/// 
/// # Arguments:
/// 
/// * `msg` - UdpMsg - .The recived message
/// * `sent_messages` - &Vec<UdpMsg> - List of messages waiting for responses
/// * `socket` -&UdpSocket  - Sending scokcet.
/// * `target_adress` - SocketAddr - reciver adress .
/// 
/// 
/// # Returns:
///
/// Returns - - .
///
pub fn handle_nak(msg: UdpMsg, sent_messages: &mut Vec<UdpMsg>, socket: &UdpSocket, target_address: SocketAddr) {
    println!("Received NAK from ID: {}", msg.header.sender_id);

    // Check if this NAK matches sent message
    if let Some(index) = sent_messages.iter().position(|m| calc_checksum(&m.data) == msg.header.checksum) {
        println!("NAK matches message with checksum: {:?}. Resending...", msg.data);

        // Resend the message
        udp_send(socket, target_address, &sent_messages[index]);
    } else {
        println!("ERROR: Received NAK with unknown checksum {:?}", msg.data);
    }
}


/// handle_new_order
/// # Arguments:
/// 
/// * `slave` - &mut Elevator - refrence to elevator.
/// * `msg` - UdpMsg - recived message.
/// * `socket` - &udpSocket - sending socket.
/// * `sender_adress` - &SocketAdress - reciving adress.
/// 
/// # Returns:
///
/// Returns -None- .
///
pub fn handle_new_order(slave: &mut Elevator, msg: UdpMsg, socket: &UdpSocket, sender_address: &SocketAddr) {
    println!("New order received ID: {}: {:?}", msg.header.sender_id, msg.data);

    if let Some(elevator)=msg.data.first(){
        for order in &elevator.queue {
            if !slave.queue.contains(&order){
                slave.queue.push(order.clone());
                println!("Order {:?} successfully added to elevator {}.", order, slave.ID);
            }else {
                println!("Order {:?} already in queue for elevator {}.", order, slave.ID);
            }
        }
    }

    udp_ack(socket, *sender_address, &msg, slave.ID);
}

/// handle_new_master
/// # Arguments:
/// 
/// * `msg` - UdpMsg - recived message.
/// * `active_elevators` - &mut Vec<elevators> - Vector of active elevators.
/// 
/// # Returns:
///
/// Returns -NONE- .
///
pub fn handle_new_master(msg: UdpMsg, active_elevators: &mut Vec<Elevator>) {
    println!("New master detected, ID: {}", msg.header.sender_id);

    // Set current master's role to Slave
    if let Some(current_master) = active_elevators.iter_mut().find(|elevator| elevator.role == Role::Master) {
        println!("Changing current master (ID: {}) to slave.", current_master.ID);
        current_master.role = Role::Slave;
    } else {
        println!("ERROR: No active master found.");
    }

    // Set new master
    if let Some(new_master) = active_elevators.iter_mut().find(|elevator| elevator.ID == msg.header.sender_id) {
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
/// * `msg` - `UdpMsg` - Message received containing the elevator ID.
/// * `active_elevators` - `&mut Vec<Elevator>` - List of currently active elevators.
/// 
/// # Returns:
///
/// Returns `true` if the elevator was added or already in the vector, otherwise `false`.
///
pub fn handle_new_online(msg: UdpMsg, active_elevators: &mut Vec<Elevator>) -> bool {
    println!("New elevator online, ID: {}", msg.header.sender_id);

    // Check if elevator is already active
    if active_elevators.iter().any(|e| e.ID == msg.header.sender_id) {
        println!("Elevator ID:{} is already active.", msg.header.sender_id);
        return true;
    }

    // Find address
    let addr = if let Some(elevator) = msg.data.first() {
        elevator.inn_address.clone()
    } else {
        println!("Error: No elevator data received.");
        return false;
    };

    //Dummy values, maybe crate a config file
    let NUM_FLOOR = 4; // find the global variable and replace this
    let inn_addr =109.108.196.139:3500;
    let out_addr =109.108.196.139:3600;
    id=1;

    // Create a new elevator instance using the provided init function
    match Elevator::init(inn_addr, out_addr, NUM_FLOOR, id) { // 
        Ok(new_elevator) => {
            active_elevators.push(new_elevator);
            println!("Added new elevator ID {} at address {}.", msg.header.sender_id, addr);
            return true;
        }
        Err(e) => {
            println!("Failed to initialize new elevator {}: {}", msg.header.sender_id, e);
            return false;
        }
    }
}


/// handle_error_worldview
/// # Arguments:
/// 
/// * `msg` - UdpMsg - .
/// * `worldview` - &mut Worldview - .
/// * `active_elevators` - &Vec<Elevator> - .
/// 
/// # Returns:
///
/// Returns - - .
///
pub fn handle_error_worldview(msg: UdpMsg, active_elevators: &mut Vec<&Elevator>) {
    println!("EROR: Worldview error reported by ID: {}", msg.header.sender_id);

    // List of orders from sender
    let missing_orders : Vec<Elevator> = msg.data.clone();

    // Compare and correct worldview based on received data
    if correct_master_worldview(master,&mut missing_orders, active_elevator) {
        println!("Worldview corrected based on report from ID: {}", msg.header.sender_id);
    } else {
        println!("ERROR: Failed to correct worldview");
    }
}

/// handle_error_offline
/// # Arguments:
/// 
/// * `msg` - UdpMsg - .
/// * `active_elevators` -  - .
/// * `active_elevators` -  - .
/// 
/// # Returns:
///
/// Returns - - .
///
pub fn handle_error_offline(msg: UdpMsg, socket: &UdpSocket,active_elevators: &mut Vec<Elevator>) {
    println!("Elevator {} went offline. Reassigning orders", msg.header.sender_id);

    let mut removed_elevator: Option<Elevator> = None;

    active_elevators.retain(|e| {
        if e.ID == msg.header.sender_id {
            removed_elevator = Some(e.clone());
            false  // Removing the elevator
        } else {
            true  // Keeping the elevator
        }
    });

    // Check if the elevator was removed
    if let Some(offline_elevator) = removed_elevator {
        println!("Removed elevator ID: {} from active list.", msg.header.sender_id);

        // Extract orders from the offline elevator
        let orders = offline_elevator.queue.clone();
        println!("Reassigning orders, if any: {:?}", orders);
        let order_ids: Vec<Order> = orders.iter().map(|order| (*order).clone()).collect();
        crate::modules::master::reassign_orders(order_ids, master, active_elevators, &mut failed_orders);
    } else {
        println!("ERROR: Elevator ID {} was not found in active list.", msg.header.sender_id);
    }
}

/// handle_error_offline
/// # Arguments:
/// 
/// * `msg` - UdpMsg - The UDP message that was recivec.
/// * `active_elevators` - &mut Vec<Elevator> - List of active elevators.
/// 
/// # Returns:
///
/// Returns - None - .
///
pub fn handle_remove_order(msg: UdpMsg, active_elevators: &mut Vec<Elevator>,failed_orders:&mut Vec<Order>) {
    println!("Removing order from ID: {}", msg.header.sender_id);

    if let Some(elevator) = active_elevators.iter_mut().find(|e| e.ID == msg.header.sender_id) {
        
        
        if let Some(elevator_from_msg) = <Vec<Elevator> as AsRef<T>>::as_ref(&msg.data).and_then(|data: &Vec<Elevator>| data.first()) {
            
            
            if let Some(order) = elevator_from_msg.queue.first() {
                
                
                if let Some(index) = elevator.queue.iter().position(|o| o == order) {
                    elevator.queue.remove(index);
                    println!("Order {:?} removed from elevator ID: {}", order, elevator.ID);
                } else {
                    println!("ERROR: Elevator ID:{} does not have order {:?}", elevator.ID, order);
                }
                
            } else {
                println!("ERROR: No orders found in the received elevator.");
            }

        } else {
            println!("ERROR: No elevator data found in the message.");
        }

    } else {
        println!("ERROR: Elevator ID:{} is not in active elevators.", msg.header.sender_id);
    }
}



/// serialize
/// Split UdpMsg into bytes for easier transmission
/// 
/// # Arguments:
/// 
/// * `msg` - &UdpMsg - .
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
    match bincode::deserialize<UdpMsg>(buffer){
        Ok(msg)=>{
        return Some(msg)},
        Err(e)=> {
            println!("Failed to dezerialise message: {}",e);
            return None;}
    }
}

/// calc_checksum
/// Calculate Checksum.
/// 
/// # Arguments:
/// 
/// * `data` &Vec<>-  - .
/// 
/// # Returns:
///
/// Returns - Vec<u8>- returns the hashed string .
///
pub fn calc_checksum(data: &Vec<Elevator>) -> Vec<u8> {
    let serialized_data = bincode::serialize(data).expect("Failed to serialize data");
    let mut hasher = Sha256::new();
    Digest::update(&mut hasher, &serialized_data);
    let hash?hasher.finalize();
    return hash.as_slice().to_vec();
}

// comp_checksum
/// Compare checksums of a message's data, with its attached checksum.
/// 
/// # Arguments:
/// 
/// * `&UdpMsg` - &UdpMsg - .
/// 
/// # Returns:
///
/// Returns - bool - returns 'true' if they match or 'false' if they dont.
///
pub fn comp_checksum(msg: &UdpMsg) -> bool {
    return calc_checksum(&msg.data) == msg.header.checksum;
}

///udp_ack
///ACK, Responds to original messag with ACK, checksum of original message is used as data to ensure which message it is responding to.
/// 
/// # Arguments:
/// 
/// * `socket` - &UdpSocket - 
/// * `target_adress` - SocketAddr - .
/// * `original_msg` - &UdpMsg - .
/// * `sender_id` - u8 - .
/// 
/// # Returns:
///
/// Returns - bool - returns 'true' if succsessful and 'false' if not.
///
pub fn udp_ack(socket: &UdpSocket, target_address: SocketAddr, original_msg: &UdpMsg, sender_id: u8) -> bool {
    let checksum = calc_checksum(&original_msg.data); // Compute checksum of original data

    let ack_msg = UdpMsg {
        header: UdpHeader {
            sender_id,
            message_type: MessageType::Ack, 
            checksum: checksum.clone(),   
        },
        data:Vec::new(), 
    };

    return udp_send(socket, target_address, &ack_msg);
}


/// udp_nak
///NAK, Responds to original messag with NAK, checksum of original message is used as data to ensure which message it is responding to.
/// 
/// # Arguments:
/// 
/// * `socket` - &UdpSocket - .
/// * `target_adress` - SocketAddr - .
/// * `original_msg` - &UdpMsg - .
/// * `sender_id` - u8 - .
///  
/// # Returns:
///
/// Returns - bool - returns 'true' if succsessful and 'false' if not.
///
pub fn udp_nak(socket: &UdpSocket, target_address: SocketAddr, original_msg: &UdpMsg, sender_id: u8) -> bool {
    let checksum = calc_checksum(&original_msg.data); // Compute checksum of original data

    let nak_msg = UdpMsg {
        header: UdpHeader {
            sender_id,
            message_type: MessageType::Nak, 
            checksum: checksum.clone(),  
        },
        data:Vec::new(), 
    };

    return udp_send(socket, target_address, &nak_msg);
}



///udp_send
///Sending UDP message.
/// 
/// # Arguments:
/// 
/// * `socket` - &UdpSocket - .
/// * `target_adress` - SocketAddr -
/// * `msg` - &UdpMsg - . .
/// 
/// # Returns:
///
/// Returns - bool - returns 'true' if sucsessfull and 'false' if not .
///
pub fn udp_send(socket: &UdpSocket, target_adress: SocketAddr, msg: &UdpMsg) -> bool {
    let data = serialize(msg);
    match socket.send_to(&data, target_adress) {
        Ok(_) => {
            println!("Message sent to: {}", target_adress);
            return true;
        }
        Err(e) => {
            eprintln!("Error sending message: {}", e);
            return false;
        }
    }
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
    let socket = UdpSocket::bind("0.0.0.0:0").expect("Failed to bind socket, broadcast");
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


//------------------------------Tests-----------------------

#[cfg(test)] // https://doc.rust-lang.orgbook/ch11-03-test-organization.html Run tests with "cargo test"
mod tests {
    use super::*;
    use std::net::{UdpSocket, SocketAddr};
    use std::sync::{Arc, Mutex};
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_serialize_deserialize() {
        let msg = UdpMsg {
            header: UdpHeader {
                sender_id: 1,
                message_type:MessageType::Ack,
                checksum: vec![0x12, 0x34],
            },
            data: vec![1, 2, 3, 4],
        };

        let serialized = serialize(&msg);
        let deserialized = deserialize(&serialized).expect("Deserialization failed");

        assert_eq!(msg.header.sender_id, deserialized.header.sender_id);
        assert_eq!(msg.header.message_id as u8, deserialized.header.message_id as u8);
        assert_eq!(msg.data, deserialized.data);
    }

    #[test]
    fn test_calc_checksum() {
        let data = vec![1, 2, 3, 4];
        let checksum = calc_checksum(&data);
        assert!(!checksum.is_empty());
    }

    #[test]
    fn test_comp_checksum() {
        let data = vec![1, 2, 3, 4];
        let checksum = calc_checksum(&data);
        let msg = UdpMsg {
            header: UdpHeader {
                sender_id: 1,
                message_type:MessageType::Ack,
                checksum: checksum.clone(),
            },
            data: data.clone(),
        };
        assert!(comp_checksum(&msg));
    }

    #[test]
    fn test_udp_send_receive() {
        let socket = UdpSocket::bind("127.0.0.1:0").expect("Failed to bind socket");
        let local_addr = socket.local_addr().expect("Failed to get socket address");

        let msg = UdpMsg {
            header: UdpHeader {
                sender_id: 1,
                message_type:MessageType::Ack,
                checksum: calc_checksum(&vec![1, 2, 3, 4]),
            },
            data: vec![1, 2, 3, 4],
        };

        let send_socket = socket.try_clone().expect("Failed to clone socket");
        let recv_socket = socket;

        let msg_clone = msg.clone();
        thread::spawn(move || {
            thread::sleep(Duration::from_millis(50));
            udp_send(&send_socket, local_addr, &msg_clone);
        });

        let received_msg = udp_receive_ensure(&recv_socket, 5, 2).expect("Failed to receive message");
        assert_eq!(msg.data, received_msg.data);
    }

    #[test]
    fn test_udp_ack_nak() {
        let socket = UdpSocket::bind("127.0.0.1:0").expect("Failed to bind socket");
        let local_addr = socket.local_addr().expect("Failed to get socket address");

        let original_msg = UdpMsg {
            header: UdpHeader {
                sender_id: 1,
                message_type:MessageType::New_Order,
                checksum: calc_checksum(&vec![5, 10, 15]),
            },
            data: vec![5, 10, 15],
        };

        let send_socket = socket.try_clone().expect("Failed to clone socket");
        let recv_socket = socket;

        thread::spawn(move || {
            thread::sleep(Duration::from_millis(50));
            udp_ack(&send_socket, local_addr, &original_msg, 2);
        });

        let received_ack = udp_receive_ensure(&recv_socket, 5, 2).expect("Failed to receive ACK");
        assert_eq!(received_ack.header.message_id, MessageType::Ack);
        assert_eq!(received_ack.data, original_msg.header.checksum);
    }

    #[test]
    fn test_udp_send_ensure() {
        let socket = UdpSocket::bind("127.0.0.1:0").expect("Failed to bind socket");
        let local_addr = socket.local_addr().expect("Failed to get socket address");

        let msg = UdpMsg {
            header: UdpHeader {
                sender_id: 1,
                message_type:MessageType::New_Order,
                checksum: calc_checksum(&vec![8, 16, 32]),
            },
            data: vec![8, 16, 32],
        };

        let mut sent_messages = Vec::new();

        let send_socket = socket.try_clone().expect("Failed to clone socket");
        let recv_socket = socket;

        let msg_clone = msg.clone();
        thread::spawn(move || {
            thread::sleep(Duration::from_millis(50)); 
            udp_ack(&send_socket, local_addr, &msg_clone, 2);
        });

        let result = udp_send_ensure(&recv_socket, &local_addr.to_string(), &msg, 3, &mut sent_messages);
        assert!(result);
    }

    #[test]
    fn test_handle_ack_nak_logic() {
        let mut sent_messages = Vec::new();
        let msg = UdpMsg {
            header: UdpHeader {
                sender_id: 1,
                message_type:MessageType::New_Order,
                checksum: calc_checksum(&vec![2, 4, 6]),
            },
            data: vec![2, 4, 6],
        };

        sent_messages.push(msg.clone());

        let ack_msg = UdpMsg {
            header: UdpHeader {
                sender_id: 2,
                message_type:MessageType::Ack,
                checksum: calc_checksum(&msg.data),
            },
            data: calc_checksum(&msg.data),
        };

        handle_ack(ack_msg, &mut sent_messages);
        assert!(!sent_messages.iter().any(|m| calc_checksum(&m.data) == ack_msg.data));
    }
}
