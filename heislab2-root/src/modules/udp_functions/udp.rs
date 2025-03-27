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
use crate::modules::master_functions::master::{give_order, best_to_worst_elevator,fix_master_issues,Role,correct_master_worldview, reassign_orders};
use crate::modules::slave_functions::slave::{update_from_worldview, check_master_failure, set_new_master};
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
            
            if !same_subnet(local_ip, sender_ip) {
                println!("Message from rejected {}(sender not in same subnet)",sender_ip);
                return None;
            }
            

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
                    MessageType::Worldview => {thread::spawn(move || {handle_worldview(passable_state, &msg_clone, udp_handler_clone)});},
                    MessageType::Ack => {thread::spawn(move || {handle_ack(&msg_clone, passable_state)});},
                    MessageType::Nak => {thread::spawn(move || {handle_nak(&msg_clone, passable_state, &sender, udp_handler_clone)});},
                    MessageType::NewOrder => {thread::spawn(move || {handle_new_order(&msg_clone, &sender, passable_state, udp_handler_clone, light_update_tx_clone,tx_clone)});},
                    MessageType::NewOnline => {thread::spawn(move ||{handle_new_online(&msg_clone, passable_state)});},
                    MessageType::ErrorWorldview => {thread::spawn(move || {handle_error_worldview(&msg_clone, passable_state)});},
                    MessageType::ErrorOffline => {handle_error_offline(&msg, passable_state, &self, tx_clone);},  // Some Error here, not sure what channel should be passed compiler says: "argument #4 of type `crossbeam_channel::Sender<Vec<Order>>` is missing"
                    MessageType::OrderComplete => {thread::spawn(move || {if !(&msg_clone.header.sender_id == &passable_state.me_id){handle_order_completed(&msg_clone, passable_state, light_update_tx_clone);}});},
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
 
    //Remove it from all orders
    let mut known_elevators_locked = state.known_elevators.lock().unwrap();
    if let Some(elevator) = known_elevators_locked.iter_mut().find(|e|e.id==completed_cab.id){
        if let Some(index) = elevator.queue.iter().position(|o|*o == completed_order){
            elevator.queue.remove(index);
            println!("Removed completed order {:?} from ID:{}",completed_order,elevator.id);
        }

    }
    let mut all_orders_locked = state.all_orders.lock().unwrap();
    if let Some(index) = all_orders_locked.iter().position(|o| *o == completed_order) {
        all_orders_locked.remove(index);
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

/// Creates a new UDP handler with a bound sockets based on this elevator
pub fn init_udp_handler(me: Cab) -> UdpHandler {

    let sender_socket = UdpSocket::bind(me.out_address).expect("Could not bind UDP socket");
    let receiver_socket = UdpSocket::bind(me.inn_address).expect("Could not bind UDP receiver socket");
    /* 
    //Linjen under er det som jeg har tullet med som burde settes tilbake
    let receiver_addr = format!("0.0.0.0:20000");
    let receiver_socket = UdpSocket::bind(receiver_addr).expect("Could not bind UDP receiver socket");
    */
    
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
pub fn handle_worldview(state: Arc<SystemState>, msg: &UdpMsg,udp_handler: Arc<UdpHandler>) {

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
pub fn handle_error_worldview(msg: &UdpMsg, state: Arc<SystemState>) {
    println!("EROR: Worldview error reported by ID: {}", msg.header.sender_id);

    // List of orders from sender
    let mut missing_orders = if let UdpData::Cabs(worldview) = &msg.data {
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
pub fn handle_error_offline(
    msg: &UdpMsg,
    state: Arc<SystemState>,
    udp_handler: &UdpHandler,
    order_update_tx: cbc::Sender<Vec<Order>>,
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

