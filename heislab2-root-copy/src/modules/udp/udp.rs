

#![allow(warnings)]

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

use crate::modules::elevator::elevator_init::Elevator;
use crate::modules::slave::slave;
use crate::modules::master::master::{Worldview,handle_multiple_masters};



//----------------------------------------------Enum
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum message_type {

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
}

//----------------------------------------------Structs
#[derive(Debug, Serialize, Deserialize, Clone)] // this is needed to serialize message
//UDP Header
pub struct UdpHeader {
    sender_id: u8,            // ID of the sender of the message.
    message_id: message_type, // ID for what kind of message it is, e.g. Button press, or Update queue.
    checksum: Vec<u8>,        // Hash of data to check message integrity.
}

#[derive(Debug, Serialize, Deserialize, Clone)] // this is needed to serialize message
                                                //UDP Message Struct
pub struct UdpMsg {
    header: UdpHeader, // Header struct containing information about the message itself
    data: Vec<u8>,     // Data so be sent.
}

//----------------------------------------------Functions

///make_Udp_msg
/// 
/// # Arguments:
/// 
/// * `elevator` - crate::modules::elevator::Elevator - . Sender
/// * `message_type` - message_type - what kind of message, check enum message_type
/// * `message` - Vec<u8> The message to be sendt
/// 
/// 
/// # Returns:
///
/// Returns - - .
///
fn make_Udp_msg(elevator: crate::modules::elevator::Elevator, message_type: message_type, message: Vec<u8>) -> UdpMsg {
    let hash = calc_checksum(&message);
    let mut overhead = UdpHeader {
        sender_id: elevator.ID,
        message_id: message_type,
        checksum: hash,
    };

    let msg = UdpMsg {
        header: overhead,
        data: message,
    };
    return msg;
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
fn udp_receive(socket: &UdpSocket, max_wait: u32, slave: &mut Elevator, me: &Elevator, worldview: &mut Worldview) -> Option<UdpMsg> {
    socket
        .set_read_timeout(Some(Duration::from_millis(max_wait)))
        .expect(&format!("Failed to set read timeout of {} ms", max_wait));

    let mut buffer = [0; 1024];

    match socket.recv_from(&mut buffer) {
        Ok((size, sender)) => {
            println!("Received message of size {} from {}", size, sender);

            if let Some(msg) = deserialize(&buffer[..size]) {
                println!("Message type: {:?}", msg.header.message_id);

                match msg.header.message_id {
                    message_type::Wordview => handle_wordview(slave, me, worldview, msg),
                    message_type::Ack => handle_ack(msg),
                    message_type::Nak => handle_nak(msg),
                    message_type::New_Order => handle_new_order(slave, msg),
                    message_type::New_Master => handle_new_master(msg),
                    message_type::New_Online => handle_new_online(msg),
                    message_type::Error_Worldview => handle_error_worldview(msg),
                    message_type::Error_Offline => handle_error_offline(msg),
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

/// 
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
fn handle_wordview(slave: &mut Elevator, me: &Elevator, worldview: &mut Worldview, msg: UdpMsg) {
    println!("Updating worldview...");
    update_from_worldview(slave, msg.data.clone());
    handle_multiple_masters(me, slave, worldview);
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
fn handle_ack(msg: UdpMsg, sent_messages: &mut Vec<UdpMsg>) {
    println!("Received ACK from ID: {}", msg.header.sender_id);

    // Check if this ACK matches sent message
    if let Some(index) = sent_messages.iter().position(|m| calc_checksum(&m.data) == msg.data) {
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
fn handle_nak(msg: UdpMsg, sent_messages: &mut Vec<UdpMsg>, socket: &UdpSocket, target_address: SocketAddr) {
    println!("Received NAK from ID: {}", msg.header.sender_id);

    // Check if this NAK matches sent message
    if let Some(index) = sent_messages.iter().position(|m| calc_checksum(&m.data) == msg.data) {
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
/// Returns - - .
///
fn handle_new_order(slave: &mut Elevator, msg: UdpMsg, socket: &UdpSocket, sender_address: &SocketAddr) {
    println!("New order received (ID: {}): {:?}", msg.header.sender_id, msg.data);

    if msg.data.is_empty(){
        println!("ERROR: Empty list recived");
        return;
    }
    for &order in msg.data.iter(){

        if receive_order(slave, order, socket, sender_address) {
            println!("Order {} successfully added to elevator {}.", order, slave.ID);
        }else {
            println!("Order {} already in queue for elevator {}.", order, slave.ID);
        }
    }
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
fn handle_new_master(msg: UdpMsg, active_elevators: &mut Vec<Elevator>) {
    println!("New master detected, ID: {}", msg.header.sender_id);

    // Set current master's role to Slave
    if let Some(current_master) = active_elevators.iter_mut().find(|elevator| elevator.status == Role::Master) {
        println!("Changing current master (ID: {}) to slave.", current_master.ID);
        current_master.status = Role::Slave;
    } else {
        println!("ERROR: No active master found.");
    }

    // Set new master
    if let Some(new_master) = active_elevators.iter_mut().find(|elevator| elevator.ID == msg.header.sender_id) {
        println!("Updating elevator ID {} to Master.", msg.header.sender_id);
        new_master.status = Role::Master;
    } else {
        println!("Error: Elevator ID {} not found in active list.", msg.header.sender_id);
    }
}

/// handle_new_online
/// Adds new online elevator to active alevators vector
/// # Arguments:
/// 
/// * `msg` - UdpMsg - .
/// * `active_elevators` - Vec<Elevator> - .
/// 
/// # Returns:
///
/// Returns - - .
///
fn handle_new_online(msg: UdpMsg, active_elevators: &mut Vec<Elevator>) {
    println!("New elevator online, ID: {}", msg.header.sender_id);

    !todo("Handle innit message from newly online elevator")
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
fn handle_error_worldview(msg: UdpMsg, worldview: &mut Worldview, active_elevators: &Vec<Elevator>) {
    println!("EROR: Worldview error reported by ID: {}", msg.header.sender_id);

    // List of orders from sender
    let reported_worldview = msg.data.clone();

    // Compare and correct worldview based on received data
    if correct_master_worldview(reported_worldview, worldview, active_elevators) {
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
fn handle_error_offline(msg: UdpMsg, active_elevators: &mut Vec<Elevator>) {
    println!("Elevator {} went offline. Reassigning orders...", msg.header.sender_id);

    // Remove the offline elevator from active list
    if let Some(index) = active_elevators.iter().position(|e| e.ID == msg.header.sender_id) {
        let offline_elevator = active_elevators.remove(index);
        println!("Removed elevator ID: {} from active list.", msg.header.sender_id);

        // Extract orders from the offline elevator
        let orders = offline_elevator.queue.clone();
        println!("Reassigning orders, if any: {:?}", orders);
        reassign_orders(orders, active_elevators);
    } else {
        println!("ERROR: Elevator ID {} was not found in active list.", msg.header.sender_id);
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
fn serialize(msg: &UdpMsg) -> Vec<u8> {
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
fn deserialize(buffer: &[u8]) -> Option<UdpMsg> {
    let deserialized_msg = bincode::deserialize(buffer).ok();
    return deserialized_msg;
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
fn calc_checksum(data: &Vec<u8>) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let hash = hasher.finalize();
    return hash.to_vec();
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
fn comp_checksum(msg: &UdpMsg) -> bool {
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
fn udp_ack(socket: &UdpSocket, target_address: SocketAddr, original_msg: &UdpMsg, sender_id: u8) -> bool {
    let checksum = calc_checksum(&original_msg.data); // Compute checksum of original data

    let ack_msg = UdpMsg {
        header: UdpHeader {
            sender_id,
            message_id: message_type::Ack, 
            checksum: checksum.clone(),   
        },
        data: checksum, 
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
fn udp_nak(socket: &UdpSocket, target_address: SocketAddr, original_msg: &UdpMsg, sender_id: u8) -> bool {
    let checksum = calc_checksum(&original_msg.data); // Compute checksum of original data

    let nak_msg = UdpMsg {
        header: UdpHeader {
            sender_id,
            message_id: message_type::Nak, 
            checksum: checksum.clone(),  
        },
        data: checksum, 
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
fn udp_send(socket: &UdpSocket, target_adress: SocketAddr, msg: &UdpMsg) -> bool {
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
fn udp_broadcast(msg: &UdpMsg) {
    let socket = UdpSocket::bind("0.0.0.0:0").expect("Failed to bind socket, broadcast");
    socket
        .set_broadcast(true)
        .expect("failed to activate broadcast");

    let msg = serialize(msg);
    let target_address = "255.255.255.255;20000";

    match socket.send_to(&msg, target_address) {
        Ok(_) => {
            println!("Broadcast successful");
        }
        Err(e) => {
            eprintln!("Error sending data: {}", e);
        }
    }
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
fn udp_send_ensure(socket: &UdpSocket, target_addr: &str, msg: &UdpMsg, max_retry: u8, sent_messages: &mut Vec<UdpMsg>) -> bool {
    let mut retries = max_retry;
    let msg_checksum = calc_checksum(&msg.data);

    // Store message in tracking list
    sent_messages.push(msg.clone());

    while retries > 0 {

        // Send the message
        if udp_send(socket, target_addr.parse().unwrap(), msg) {
            println!("Sent message {} to {}", msg.header.message_id as u8, target_addr);
        } else {
            println!("Error sending message, retrying...");
        }

        // Wait for a response
        let mut buffer = [0; 1024];
        match socket.recv_from(&mut buffer) {
            Ok((size, rec_addr)) if rec_addr.to_string() == target_addr => {
                if let Some(response_msg) = deserialize(&buffer[..size]) {
                    match response_msg.header.message_id {
                        message_type::Ack => {
                            if response_msg.data == msg_checksum {
                                println!("ACK received for message {}", msg.header.message_id as u8);
                                sent_messages.retain(|m| calc_checksum(&m.data) != msg_checksum);
                                return true; 
                            } else {
                                println!("ERROR: Received ACK wrong checksum!");
                            }
                        }
                        message_type::Nak => {
                            if response_msg.data == msg_checksum {
                                println!("NAK received, resending message...");
                            } else {
                                println!("ERROR: Received NAK with unknown checksum!");
                            }
                        }
                        _ => {
                            println!("ERROR: Unexpected message type received: {:?}", response_msg.header.message_id);
                        }
                    }
                }
            }
            Err(e) => {
                println!("No response received befor timeout, retrying... [{} retries left]", retries);
            }
        }

        retries -= 1;
    }

    println!("Failed to send message {} after {} retries.", msg.header.message_id as u8, max_retry);
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
fn udp_receive_ensure(socket: &UdpSocket, max_wait: u8, receiver_id: u8) -> Option<UdpMsg> {
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

