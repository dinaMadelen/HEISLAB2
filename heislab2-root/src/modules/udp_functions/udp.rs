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
use std::time::{Duration,Instant, SystemTime};              // https://doc.rust-lang.org/std/time/struct.Duration.html
// use std::thread::sleep;                      // https://doc.rust-lang.org/std/thread/fn.sleep.html
use std::sync::{Mutex,Arc};                     // https://doc.rust-lang.org/std/sync/struct.Mutex.html
use crossbeam_channel as cbc;
use std::thread;

use crate::modules::order_object::order_init::Order;
use crate::modules::elevator_object::elevator_init::SystemState;
use crate::modules::cab_object::cab::Cab;
use crate::modules::master_functions::master::{give_order, best_to_worst_elevator,handle_multiple_masters,Role,correct_master_worldview, reassign_orders};
use crate::modules::slave_functions::slave::update_from_worldview;
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
pub struct Worldview{
    pub live: Vec<Cab>,
    pub dead: Vec<Cab>
}

#[derive(Debug, Serialize, PartialEq, Deserialize, Clone)]
pub enum UdpData {
    Checksum(u32),
    Cabs(Vec<Cab>),
    Cab(Cab),
    Orders(Vec<Order>),
    Order(Order),
    Worldview(Worldview),
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


        udp_broadcast(message);

        let active_elevators_locked = state.active_elevators.lock().unwrap();
        if active_elevators_locked.len() == 1 && active_elevators_locked[0].id == state.me_id {

            //Mark message as recvied 
            let mut sent_messages_locked = state.sent_messages.lock().unwrap();
            if let Some(entry) = sent_messages_locked.iter_mut().find(|m| m.message_hash == message.header.checksum) {
                entry.all_confirmed = true;
            }
            println!("This is the only elevator in system, skipping ACK wait.");
        }
        drop(active_elevators_locked);
        

        // Check if there are missing acks
        if !confirm_recived(message, state) {
    
            while retries > 0 {
    
                println!("Remaining retries: {}", retries);
                retries -= 1;
    
                std::thread::sleep(Duration::from_millis(30));
    
                //Lock mutexes
                let sent_messages_locked = state.sent_messages.lock().unwrap();
                let active_elevators_locked = state.active_elevators.lock().unwrap();
    
                if let Some(waiting) = sent_messages_locked.iter().find(|m| m.message_hash == message.header.checksum) {
                    let expected_ids: Vec<u8> = active_elevators_locked.iter().map(|e| e.id).collect();
                    let missing: Vec<u8> = expected_ids.into_iter().filter(|id| !waiting.responded_ids.contains(id)).collect();
                
    
                    for elevator_id in &missing {
                        if let Some(elevator) = active_elevators_locked.iter().find(|e| e.id == *elevator_id){
    
                            let target_address = &elevator.inn_address;
                            self.send(&target_address, message);
                        }
                    }
                }
    
                drop(sent_messages_locked);
                drop(active_elevators_locked);

    
                if confirm_recived(&message, state){
                    println!("Successfully acknowledged by all elevators.");
                    return true;
                }
            
                println!("Missing acknowledgments.");
    
            }
        }else{
            println!("Successfully acknowledged by all elevators.");
            return true;
        }
    
        // Remove waiting for Acks from state
        let mut sent_messages_locked = state.sent_messages.lock().unwrap();
        sent_messages_locked.retain(|m| m.message_hash != message.header.checksum);
        drop(sent_messages_locked);
    
        println!("Failed to deliver order after {} retries.", retries);
        return false;
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
    pub fn receive(self: Arc<Self>, max_wait: u32, state: &Arc<SystemState>, order_update_tx: cbc::Sender<Vec<Order>>, light_update_tx: cbc::Sender<Vec<Order>>) -> Option<UdpMsg> {

        //Lock socket from udp.handler
        let sock = self.receiver_socket.lock().expect("Failed to lock receiver socket");

        //Set timeout for reciving messages (How long are you willing to wait)
        sock.set_read_timeout(Some(Duration::from_millis(max_wait as u64))).expect("Failed to set timeout for socket");
        let mut buffer = [0; 1024];


        //Find IP
        let local_ip = sock.local_addr().expect("Failed to get local address").ip();
        drop(sock); 
        

        loop{

            let sock = self.receiver_socket.lock().expect("Failed to lock receiver socket");
            // Receive data
            let (size, sender) = match sock.recv_from(&mut buffer) {
                Ok(res) => res,
                Err(ref e) if (e.kind() == std::io::ErrorKind::WouldBlock || e.kind() == std::io::ErrorKind::TimedOut) => {
                    // Ignore the error if it's just a timeout
                    return None;
                }
                Err(e) => {
                    println!("Failed to receive message: {}", e);
                    return None;
                }
            };
            drop(sock); 

            let sender_ip = sender.ip();

            //Check that the sender is from the same subnet, we dont want any outside messages
            //UNCOMMENT THIS
            /* 
            if !same_subnet(local_ip, sender_ip) {
                println!("Message from rejected {}(sender not in same subnet)",sender_ip);
                return None;
            }
            */

            println!("Received message of size {} from {}", size, sender);
            // Identify Messagetype and handle appropriatly
            if let Some(msg) = msg_deserialize(&buffer[..size]) {
                println!("Message type: {:?}", msg.header.message_type);

                let passable_state = Arc::clone(state);
                let udp_handler_clone = Arc::clone(&self);
                let msg_clone = msg.clone();
                let tx_clone = order_update_tx.clone();
                let light_update_tx_clone = light_update_tx.clone();
                


                match msg.header.message_type{
                    MessageType::Worldview => {thread::spawn(move || {handle_worldview(passable_state, &msg_clone)});},
                    MessageType::Ack => {thread::spawn(move || {handle_ack(&msg_clone, passable_state)});},
                    MessageType::Nak => {thread::spawn(move || {handle_nak(&msg_clone, passable_state, &sender, udp_handler_clone)});},
                    MessageType::NewOrder => {thread::spawn(move || {handle_new_order(&msg_clone, &sender, passable_state, udp_handler_clone, light_update_tx_clone,tx_clone)});},
                    MessageType::NewOnline => {thread::spawn(move || {handle_new_online(&msg_clone, passable_state)});},
                    MessageType::ErrorWorldview => {thread::spawn(move || {handle_error_worldview(&msg_clone, passable_state)});},
                    MessageType::ErrorOffline => {handle_error_offline(&msg, passable_state, &self, tx_clone);},  // Some Error here, not sure what channel should be passed compiler says: "argument #4 of type `crossbeam_channel::Sender<Vec<Order>>` is missing"
                    MessageType::OrderComplete => {thread::spawn(move || {handle_remove_order(&msg_clone, passable_state, light_update_tx_clone)});},
                    MessageType::NewRequest => {thread::spawn(move || {handle_new_request(&msg_clone,passable_state, udp_handler_clone,tx_clone, light_update_tx_clone)});},
                    MessageType::NewMaster => {thread::spawn(move ||{ handle_new_master(&msg_clone, passable_state)});},
                    MessageType::ImAlive => {thread::spawn(move ||{ handle_im_alive(&msg_clone, passable_state)});},
                    _ => println!("Unreadable message received from {}", sender),
                };
                //return Some(msg);
            } else {
                println!("Failed to deserialize message from {}", sender);
                //return None;
            }
        }
    }
}

pub fn handle_im_alive(msg: &UdpMsg, state: Arc<SystemState>){
    //Extract updated cab from message
    let updated_cab = if let UdpData::Cab(cab) = &msg.data{
        cab.clone()
    }else{
        println!("Couldnt read ImAlive message");
        return;
    };

    //Replace the old cab struct with the updated cab struct
    let mut active_elevators_locked = state.active_elevators.lock().unwrap();
    if let Some(sender_elevator) = active_elevators_locked.iter_mut().find(|e| e.id == msg.header.sender_id){
        println!("Updating alive elevator");
        sender_elevator.merge_with(&updated_cab); 
        sender_elevator.last_lifesign = SystemTime::now();
        //Update last lifesign of that elevator

    } else {
        println!("Sender elevator not in active elevators");
        //Send a NewOnline message with that cab
        let mut dead_elevators_locked = state.dead_elevators.lock().unwrap();

        println!("Seardching dead elevators");
        if let Some(pos) = dead_elevators_locked.iter().position(|e| e.id == msg.header.sender_id) {
            // Remove from dead
            let resurrected_elevator = dead_elevators_locked.remove(pos);
            // Push to active
            active_elevators_locked.push(resurrected_elevator);
        } else {
            println!("Sender elevator not in dead or active elevators, pushing to active elevators");
            active_elevators_locked.push(updated_cab);
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
    drop(all_orders_locked); // This one sounds scary

    //Check if this elevator is master 
    let master_id_clone = state.master_id.lock().unwrap().clone();
    let is_master = state.me_id == master_id_clone;
    //Lock active_elevators

    //IF New Request is CAB order
    if new_order.order_type == CAB{
        
        // Lock the active elevators and find the elevator that matches the sender id.
        let mut active_elevators_locked = state.active_elevators.lock().unwrap();
        if let Some(sender_elevator) = active_elevators_locked.iter_mut().find(|e| e.id == msg.header.sender_id) {
            sender_elevator.queue.push(new_order.clone());
            if sender_elevator.id == state.me_id{
                light_update_tx.send(sender_elevator.queue.clone()).unwrap();
            }
            println!("Entered call type cab");
            if is_master {
                // Capture necessary data (elevator id) before dropping the lock.
                let elevator_id = sender_elevator.id;
                // Lock is dropped here when the block ends.
                drop(active_elevators_locked);
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
            let active_elevators_locked = state.active_elevators.lock().unwrap();
            let best_elevators = best_to_worst_elevator(&new_order, &*active_elevators_locked);
            drop(active_elevators_locked);

            let best_elevator = match best_elevators.first() {
                Some(elevator) => {
                    println!("Assigning new hallcall to {:?}", elevator);
                    elevator
                }
                None => {
                    println!("No available elevator to assign the order.");
                    return; // Or handle the situation appropriately.
                }
            };

            let give_order_success = give_order(*best_elevator, vec![&new_order], &state, &udp_handler);
            {
                let mut active_elevators_locked = state.active_elevators.lock().unwrap();
                //If not all acs are recieved, give order to self
                if !give_order_success{active_elevators_locked.get_mut(0).unwrap().queue.push(new_order.clone());};
            }
        }       
    }
    order_update_tx.send(vec![new_order.clone()]).unwrap();
}

/// Creates a new UDP handler with a bound sockets based on this elevator
pub fn init_udp_handler(me: Cab) -> UdpHandler {

    let sender_socket = UdpSocket::bind(me.out_address).expect("Could not bind UDP socket");
    /*let receiver_socket = UdpSocket::bind(me.inn_address).expect("Could not bind UDP receiver socket");
    */
    //Linjen under er det som jeg har tullet med som burde settes tilbake
    let receiver_socket = UdpSocket::bind("0.0.0.0:20000").expect("Could not bind UDP receiver socket");
    
    
    sender_socket.set_nonblocking(true).expect("Failed to set non-blocking mode");
    receiver_socket.set_nonblocking(true).expect("Failed to set non-blocking mode");

    //Turn sockets into mutexes
    let sender_socket = Arc::new(Mutex::new(sender_socket));
    let receiver_socket = Arc::new(Mutex::new(receiver_socket));
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
pub fn handle_worldview(state: Arc<SystemState>, msg: &UdpMsg) {
    println!("Updating worldview...");

    //Update last lifesign and last worldview
    let mut last_lifesign_locked = state.lifesign_master.lock().unwrap();
    *last_lifesign_locked = Instant::now();
    drop(last_lifesign_locked);

    let mut new_worldview = state.last_worldview.lock().unwrap();
    *new_worldview = msg.clone();
    drop(new_worldview);
    
    
    
    let worldview = if let UdpData::Worldview(worldview) = &msg.data{
        worldview
    }
    else{
        println!("Wrong data in message for worldview");
        return;
    };

    update_from_worldview(&state, &worldview);
    /* let active_elevators: Vec<Cab> = {
    let active_elevators_locked = state.active_elevators.lock().unwrap();
    active_elevators_locked.clone() */ 
     
    //not used
    //generate_worldview(&active_elevators);


    handle_multiple_masters(&state, &msg.header.sender_id);
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
        let active_elevators_locked = state.active_elevators.lock().unwrap();

        //Check that all active elevatos have responded 
        for elevator in active_elevators_locked.iter(){
            if !waiting.responded_ids.contains(&elevator.id){
                println!("Still missing confirmations for elevaotr ID:{}", elevator.id);
                all_confirmed = false;
            }

        }
        drop(active_elevators_locked);

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
    let mut active_elevators_locked = state.active_elevators.lock().unwrap(); 

    //Find elevator with mathcing ID and update queue
    if let Some(update_elevator) = active_elevators_locked.iter_mut().find(|e| e.id == elevator_id){
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
    //Send Ack to sender

    let msg1 = UdpData::Checksum(1234);
    let encoded = bincode::serialize(&msg1).unwrap();
    println!("Sender enum tag: {}", encoded[0]); // Should be 0
    return udp_ack(*sender_address, &msg, elevator.id, &udp_handler);
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
pub fn handle_new_master(msg: &UdpMsg, state: Arc<SystemState>) {
    println!("New master detected, ID: {}", msg.header.sender_id);

    // Set current master's role to Slave
    let mut active_elevators_locked = state.active_elevators.lock().unwrap();
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
pub fn handle_new_online(msg: &UdpMsg, state: Arc<SystemState>) -> bool {
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
        last_lifesign: SystemTime::now(),
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
pub fn handle_error_worldview(msg: &UdpMsg, state: Arc<SystemState>) {
    println!("EROR: Worldview error reported by ID: {}", msg.header.sender_id);

    // List of orders from sender
    let mut missing_orders = if let UdpData::Worldview(worldview) = &msg.data {
        worldview.clone()
    } else {
        println!("ERROR: Expected UdpData::Cabs but got something else");
        return;
    };

    // Compare and correct worldview based on received data
    if correct_master_worldview(&mut missing_orders, &state) {
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
pub fn handle_error_offline(msg: &UdpMsg,state: Arc<SystemState> ,udp_handler: &UdpHandler, order_update_tx: cbc::Sender<Vec<Order>>) {
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
        reassign_orders(&order_ids, &state ,udp_handler, order_update_tx);
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
    let mut active_elevators_locked = state.active_elevators.lock().unwrap();

    //Check for correct elevator in active elevators
    if let Some(elevator) = active_elevators_locked.iter_mut().find(|e| e.id == remove_id) {
    
        if let Some(order) = elevator_from_msg.queue.first() {
                    
            if let Some(index) = elevator.queue.iter().position(|o| o == order) {
                elevator.queue.remove(index);
                println!("Order {:?} removed from elevator ID: {}", order, elevator.id);
                if elevator.id == state.me_id{
                    light_update_tx.send(elevator.queue.clone()).unwrap();
                }
                
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
            println!("Deserialized msg type: {:?},Deserialized msg data: {:?}", msg.header.message_type, msg.data);
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
        (MessageType::Worldview, UdpData::Worldview(_)) => true,
        (MessageType::OrderComplete, UdpData::Cab(_)) => true,
        (MessageType::NewRequest, UdpData::Order(_)) => true,
        (MessageType::ImAlive, UdpData::Cab(_)) => true,
        (MessageType::ErrorWorldview, UdpData::Cabs(_)) => true,
        (MessageType::ErrorOffline, UdpData::Cab(_)) => true,
        (MessageType::NewMaster, UdpData::Cab(_)) => true,
        (MessageType::NewOnline, UdpData::Cab(_)) => true,
        (MessageType::Ack, UdpData::Checksum(_)) => true,
        (MessageType::Nak, UdpData::Checksum(_)) => true,
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


//Check if the subnet (not full ip) matches.
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
            //Remove message
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

