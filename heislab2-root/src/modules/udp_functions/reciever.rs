// public crates
use crossbeam_channel as cbc;

use std::{
    net::{SocketAddr, IpAddr, Ipv4Addr},
    thread,
    time::{Duration,Instant, SystemTime},
    sync::Arc
};

// project crates
use crate::modules::{
    io::io_init::IoChannels, 
    udp_functions::udp::*,
    udp_functions::handlers::*,
    system_status::SystemState,
    order_object::order_init::*,
};

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
                    MessageType::ErrorWorldview => {thread::spawn(move || {handle_error_worldview(&msg_clone, passable_state, &udp_handler_clone, &tx_clone)});},
                    MessageType::ErrorOffline => {handle_error_offline(&msg, passable_state, &self, &tx_clone);},  // Some Error here, not sure what channel should be passed compiler says: "argument #4 of type `crossbeam_channel::Sender<Vec<Order>>` is missing"
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
