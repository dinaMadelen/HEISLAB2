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
#[allow(non_camel_case_types)]

//----------------------------------------------Imports
use std::net::{IpAddr,SocketAddr, UdpSocket};
//use std::ops::DerefMut;                        // https://doc.rust-lang.org/std/net/struct.UdpSocket.html       
use serde::{Deserialize, Serialize};            // https://serde.rs/impl-serialize.html         //Add to Cargo.toml file, Check comment above
                                                // https://docs.rs/serde/latest/serde/ser/trait.Serialize.html#tymethod.serialize
use bincode;                                    // https://docs.rs/bincode/latest/bincode/      //Add to Cargo.toml file, Check comment above
use crc32fast::Hasher;                          // Add to Cargo.toml file, Check comment above  //Add to Cargo,toml Smaller but less secure hash than Sha256, this is 4Bytes while Sha256 is 32Bytes
//use sha2::{Digest, Sha256};                     // https://docs.rs/sha2/latest/sha2/            //Add to Cargo.toml file, Check comment above
use std::time::{Duration};              // https://doc.rust-lang.org/std/time/struct.Duration.html
// use std::thread::sleep;                      // https://doc.rust-lang.org/std/thread/fn.sleep.html
use std::sync::{Mutex,Arc};                     // https://doc.rust-lang.org/std/sync/struct.Mutex.html

use crate::modules::order_object::order_init::Order;
use crate::modules::elevator_object::elevator_init::SystemState;
use crate::modules::cab_object::cab::Cab;
use crate::modules::system_status::WaitingConfirmation;

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
    pub sender_id: u8,             // ID of the sender of the message.
    pub message_type: MessageType, // ID for what kind of message it is, e.g. Button press, or Update queue.
    pub checksum: u32,         // Hash of data to check message integrity.
}

#[derive(Debug, Serialize, PartialEq, Deserialize, Clone)] // this is needed to serialize message
//UDP Message Struct
pub struct UdpMsg {
    pub header: UdpHeader,       // Header struct containing information about the message itself
    pub data: UdpData,        // Data so be sent.
}
#[derive (Clone, Debug)]
pub struct UdpHandler {
    pub sender_socket: Arc<Mutex<UdpSocket>>,
    pub receiver_socket: Arc<Mutex<UdpSocket>>,
}

#[derive(Debug, Serialize, PartialEq, Deserialize, Clone)]
pub enum UdpData {
    Checksum(u32),
    Cabs(Vec<Cab>),
    Cab(Cab),
    Orders(Vec<Order>),
    Order(Order),
}



//----------------------------------------------Functions


impl UdpHandler {

    /// Sends a UDP message
    pub fn send(&self, target_address: &SocketAddr, msg: &UdpMsg) -> bool {

        let data = msg_serialize(msg);
        let sock = self.sender_socket.lock().expect("Faild to lock socket for sending");

        match sock.send_to(&data, target_address) {
            Ok(_) => {
                println!("Message type:{:?} sent to: {}", msg.header.message_type, target_address);
                return true;
            }
            Err(e) => {
                eprintln!("Error sending message: {}", e);
                return false;
            }
        };
    }


    pub fn ensure_broadcast(&self, message:&UdpMsg, state: &Arc<SystemState>, max_retries:u8) -> bool {
        let mut retries = max_retries;

        // Add check for acks
        let confirmation = WaitingConfirmation {message_hash: message.header.checksum, responded_ids: vec![state.me_id],   all_confirmed: false,};
        let mut sent_messages_locked = state.sent_messages.lock().unwrap();
        sent_messages_locked.push(confirmation);
        drop(sent_messages_locked);

        let known_elevators_locked = state.known_elevators.lock().unwrap();
        for elevator in known_elevators_locked.iter(){
                            self.send(&elevator.inn_address, &message);
        }
        drop(known_elevators_locked);

        let known_elevators_locked = state.known_elevators.lock().unwrap();
        if known_elevators_locked.len() == 1 && known_elevators_locked[0].id == state.me_id {

            //Mark message as recvied 
            let mut sent_messages_locked = state.sent_messages.lock().unwrap();
            if let Some(entry) = sent_messages_locked.iter_mut().find(|m| m.message_hash == message.header.checksum) {
                entry.all_confirmed = true;
            }
            println!("This is the only elevator in system, skipping ACK wait.");
            return false;
        }
        drop(known_elevators_locked);
        

        // Check if we already have all ACKs.
    if !confirm_recived(message, state) {
        let mut last_expected_ids: Vec<u8> = Vec::new();
        let mut last_waiting: Option<WaitingConfirmation> = None;

        while retries > 0 {
            println!("Remaining retries: {}", retries);
            retries -= 1;
            std::thread::sleep(Duration::from_millis(30));

            // Clone the current confirmation list snapshot.
            let sent_messages_snapshot = state.sent_messages.lock().unwrap().clone();
            let known_elevators = state.known_elevators.lock().unwrap();

            if let Some(waiting) = sent_messages_snapshot
                .iter()
                .find(|m| m.message_hash == message.header.checksum)
            {
                // Only consider elevators marked as alive.
                let expected_ids: Vec<u8> = known_elevators.iter().filter(|e| e.alive).map(|e| e.id).collect();
                let missing: Vec<u8> = expected_ids.iter().cloned().filter(|id| !waiting.responded_ids.contains(id)).collect();

                last_expected_ids = expected_ids;
                last_waiting = Some(waiting.clone());

                // Resend the message only to the ones that have not acked.
                for elevator_id in missing.iter() {
                    if let Some(elevator) = known_elevators.iter().find(|e| e.id == *elevator_id) {
                        self.send(&elevator.inn_address, message);
                    }
                }
            }
            // Check if we now have all ACKs.
            if confirm_recived(message, state) {
                println!("Successfully acknowledged by all elevators.");
                return true;
            }
        }

        // After all retries have been exhausted, mark any missing elevators as dead.
        if let Some(waiting) = last_waiting {
            let missing: Vec<u8> = last_expected_ids.into_iter().filter(|id| !waiting.responded_ids.contains(id)).collect();
            if !missing.is_empty() {
                let mut known_elevators = state.known_elevators.lock().unwrap();
                for elevator in known_elevators.iter_mut() {
                    if missing.contains(&elevator.id) {
                        elevator.alive = false;
                        println!("Elevator {} marked as dead after all retries failed.", elevator.id);
                    }
                }
            }
        }
        return false;

    } else {
        println!("Successfully acknowledged by all elevators.");
        return true;
    }
    }


}

/// Creates a new UDP handler with a bound sockets based on this elevator
pub fn initialize_udp_handler(me_clone: Cab) -> Arc<UdpHandler> {

    let sender_socket = UdpSocket::bind(me_clone.out_address).expect("Could not bind UDP socket");
    let receiver_socket = UdpSocket::bind(me_clone.inn_address).expect("Could not bind UDP receiver socket");
    
    sender_socket.set_nonblocking(true).expect("Failed to set non-blocking mode");
    receiver_socket.set_nonblocking(true).expect("Failed to set non-blocking mode");

    //Turn sockets into mutexes
    let sender_socket = Arc::new(Mutex::new(sender_socket));
    let receiver_socket = Arc::new(Mutex::new(receiver_socket));

    Arc::new(UdpHandler{sender_socket,receiver_socket})
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
/// * `data` - &Vec<elevator> - refrence to list of elevators.
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
/// Returns - None - .
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


//Check if the subnet matches.
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

