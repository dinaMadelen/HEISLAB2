use crate::modules::order_object::order_init::Order;
use crate::modules::cab_object::cab::Cab;
use crate::modules::udp_functions::udp::UdpMsg;

use std::sync::{Arc, Mutex};
use std::time::Instant;

#[derive(Clone, Debug)]
pub struct SystemState {
    pub me_id : u8,
    pub master_id: Arc<Mutex<u8>>,
    pub last_lifesign: Arc<Mutex<Instant>>,
    pub last_worldview: Arc<Mutex<UdpMsg>>,
    pub active_elevators: Arc<Mutex<Vec<Cab>>>,
    pub dead_elevators: Arc<Mutex<Vec<Cab>>>,
    pub all_orders: Arc<Mutex<Vec<Order>>>,
    pub sent_messages: Arc<Mutex<Vec<WaitingConfirmation>>>,
}

#[derive(Clone, Debug)]
pub struct WaitingConfirmation {
    pub message_hash: u32,
    pub responded_ids: Vec<u8>,
    pub all_confirmed: bool,
}

