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
//! - 'msg_serialize'     serializes UDP messages for transmission.           
//! - 'msg_deserialize'    deserializes transmitted udp messages.
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
#[allow(non_camel_case_types)]                  // https://doc.rust-lang.org/std/net/struct.UdpSocket.html       
use serde::{Deserialize, Serialize};            // https://serde.rs/impl-serialize.html         //Add to Cargo.toml file, Check comment above
                                                // https://docs.rs/serde/latest/serde/ser/trait.Serialize.html#tymethod.serialize
use bincode;                                    // https://docs.rs/bincode/latest/bincode/      //Add to Cargo.toml file, Check comment above
use crc32fast::Hasher;                          // Add to Cargo.toml file, Check comment above  //Add to Cargo,toml Smaller but less secure hash than Sha256, this is 4Bytes while Sha256 is 32Bytes
use std::sync::Arc;                             // https://doc.rust-lang.org/std/sync/struct.Mutex.html

use std::net::{SocketAddr,IpAddr,UdpSocket};


use crate::modules::order_object::order_init::Order;
use crate::modules::elevator_object::elevator_init::SystemState;
use crate::modules::cab_object::cab::Cab;

use crate::modules::udp_functions::udp_handler_init::*;


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
    ImAlive,
}

//----------------------------------------------Structs
#[derive(Debug, Serialize, PartialEq, Deserialize, Clone)] // this is needed to serialize message
//UDP Header
pub struct UdpHeader {
    pub sender_id: u8,              // ID of the sender of the message.
    pub message_type: MessageType,  // ID for what kind of message it is, e.g. Button press, or Update queue.
    pub checksum: u32,               // Hash of data to check message integrity.
}

#[derive(Debug, Serialize, PartialEq, Deserialize, Clone)] // this is needed to serialize message
//UDP Message Struct
pub struct UdpMsg {
    pub header: UdpHeader,          // Header struct containing information about the message itself
    pub data: UdpData,              // Data so be sent.
}


#[derive(Debug, Serialize, PartialEq, Deserialize, Clone)]
pub enum UdpData {
    Checksum(u32),
    Cabs(Vec<Cab>),
    Cab(Cab),
    Orders(Vec<Order>),
    Order(Order),
}


///make_udp_msg
/// 
/// # Arguments:
/// * `sender_id` - u8 - Id of sender
/// * `message_type` - MessageType - what kind of message, check enum MessageType
/// * `message` - Vec<Cab> The message to be sendt
/// 
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
/// msg_serialize
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
pub fn msg_serialize(msg: &UdpMsg) -> Vec<u8> {
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
/// Returns - Option<UdpMsg>- .returns either the deserialized message or none
///
pub fn msg_deserialize(buffer: &[u8]) -> Option<UdpMsg> {
    match bincode::deserialize::<UdpMsg>(buffer) {
        Ok(msg) => {
            // println!("Deserialized msg type: {:?},Deserialized msg data: {:?}", msg.header.message_type, msg.data);
            if data_valid_for_type(&msg) {
                return Some(msg);
            } else {
                println!("Invalid data messagetype");
                return None;
            }
        }
        Err(e) => {
            println!("Failed to deserialize message: {}", e);
            return None;
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
        (MessageType::NewOrder, UdpData::Cab(_)) => true,
        (MessageType::Worldview, UdpData::Cabs(_)) => true,
        (MessageType::OrderComplete, UdpData::Cab(_)) => true,
        (MessageType::NewRequest, UdpData::Order(_)) => true,
        (MessageType::ImAlive, UdpData::Cab(_)) => true,
        (MessageType::ErrorWorldview, UdpData::Cabs(_)) => true,
        (MessageType::ErrorOffline, UdpData::Cab(_)) => true,
        (MessageType::NewMaster, UdpData::Cab(_)) => true,
        (MessageType::NewOnline, UdpData::Cab(_)) => true,
        (MessageType::Ack, UdpData::Checksum(_)) => true,
        (MessageType::Nak, UdpData::Checksum(_)) => true,
        (MessageType::RemoveOrder, UdpData::Cabs(_)) => true,
        _ => false,
    }
}

/// calc_checksum
/// Calculate Checksum.
/// 
/// # Arguments:
/// 
/// * `data` - &UdpData - refrenence to data to be send, see UdpData
/// 
/// # Returns:
///
/// Returns - Vec<u8>- returns the hashed string .
///
pub fn calc_checksum(data: &UdpData) -> u32 {
    let serialized_data = bincode::serialize(data).expect("Failed to serialize data");
    let mut hasher = Hasher::new();
    hasher.update(&serialized_data);
    return hasher.finalize();
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

    let new_data =UdpData::Checksum(original_msg.header.checksum);
    let checksum = calc_checksum(&new_data);

    let ack_msg = UdpMsg {
        header: UdpHeader {
            sender_id,
            message_type: MessageType::Ack, 
            checksum: checksum,   
        },
        data: UdpData::Checksum(original_msg.header.checksum), 
    };
    println!("Sending ACK with data: {:?}", ack_msg.data);
    
    return udp_handler.send(&target_address, &ack_msg);
}

//------------------------MOVE TO HANDLER STRUCT
/// udp_nak
///NAK, Responds to original messag with NAK, checksum of original message is used as data to ensure which message it is responding to.
/// 
/// # Arguments:
/// 
/// * `target_adress` - SocketAddr - .the socket you are sending to
/// * `original_msg` - &UdpMsg - . the original message
/// * `sender_id` - u8 - id of cab
/// * `udp_handler` -&UdpHandler- udphandler for sending messages over udp
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
            checksum: calc_checksum(&UdpData::Checksum(original_msg.header.checksum)),
        },
        data: UdpData::Checksum(original_msg.header.checksum), 
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
/// Returns - bool - `true` if succesful else `false`.
///
pub fn udp_broadcast(msg: &UdpMsg) -> bool {
    let socket = UdpSocket::bind("0.0.0.0:0").expect("Failed to bind socket for broadcast");
    socket
        .set_broadcast(true)
        .expect("failed to activate broadcast");

    let msg = msg_serialize(msg);
    let target_address = "255.255.255.255:20000";

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

//same subnet
//Check if the sender is from the same subnet.
/// 
/// # Arguments:
/// 
/// * `local` - IpAddr - ip of your computer.
/// * `remote` - IpAddr - if of sender
/// 
/// # Returns:
///
/// Returns - bool - `true` if succesful same subnet.
///
pub fn same_subnet(local: IpAddr, remote: IpAddr) -> bool {

    match (local, remote) {
        // Check that both are IPv4
        (IpAddr::V4(local_ip), IpAddr::V4(remote_ip)) => {
            //Split IP adress into array of bytes
            let local = local_ip.octets();
            let remote = remote_ip.octets();
            
            // Check that each byte in the subnet matches its corresponding byte in the other adress
            return local[0] == remote[0] && local[1] == remote[1] && local[2] == remote[2];
        }
        _ => return false
    }
}


//confirm recived
//Check if all acks has been recived for a Udpmessage
/// 
/// # Arguments:
/// 
/// * `msg` - &UdpMsg - refrence to original message
/// * `state` - &Arc<SystemState> - Current systemstate
/// 
/// # Returns:
///
/// Returns - bool - `true` if succesful same subnet.
///
pub fn confirm_recived(msg:&UdpMsg, state: &Arc<SystemState>) -> bool {
    
    let checksum = msg.header.checksum;

    let mut sent_messages_locked = state.sent_messages.lock().unwrap();
    if let Some(waiting_for_confirmation) = sent_messages_locked.iter().find(|m|m.message_hash==checksum){

        if waiting_for_confirmation.all_confirmed{
            println!("Confirmation checked message with checksum {}, all acks recvied", checksum);
            //Remove messages 
            sent_messages_locked.retain(|m| m.message_hash != checksum);
            return true;
        }else{
            println!("Confirmation checked message with checksum {}, Not recived all acks", checksum);
            return false;
        }
    }
    println!("Checksum:{} not in sent messages", checksum);
    return false;
}



