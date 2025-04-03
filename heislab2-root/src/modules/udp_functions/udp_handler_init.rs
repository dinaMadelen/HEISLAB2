#[allow(unused_imports)]
#[allow(unused_variables)]
#[allow(non_camel_case_types)]

//----------------------------------------------Imports
use std::net::{IpAddr,SocketAddr, UdpSocket};
use std::time::Duration;              // https://doc.rust-lang.org/std/time/struct.Duration.html
use std::sync::{Mutex,Arc};                     // https://doc.rust-lang.org/std/sync/struct.Mutex.html
use crossbeam_channel as cbc;
use std::thread;

use crate::modules::udp_functions::message_handlers::*;
use crate::modules::udp_functions::udp::*;

use crate::modules::order_object::order_init::Order;
use crate::modules::elevator_object::elevator_init::SystemState;
use crate::modules::cab_object::cab::Cab;
use crate::modules::system_status::WaitingConfirmation;

pub use crate::modules::elevator_object::*;
pub use elevator_init::Elevator;
pub use alias_lib::{HALL_DOWN, HALL_UP,CAB, DIRN_DOWN, DIRN_UP, DIRN_STOP};
use local_ip_address::local_ip;

#[derive (Clone, Debug)]
pub struct UdpHandler {
    pub sender_socket: Arc<Mutex<UdpSocket>>,
    pub receiver_socket: Arc<Mutex<UdpSocket>>,
}


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


impl UdpHandler {

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
        let local_ip = local_ip().unwrap();
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
            std::thread::sleep(Duration::from_millis(50));

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
