use crate::modules::order_object::order_init::Order;
use crate::modules::cab::cab::Cab;
use crate::modules::udp::udp::UdpMsg;

use std::sync::{Arc, Mutex};
use std::time::Instant;

pub struct SystemState {
    pub me_id : u8,
    pub master_id: Arc<Mutex<u8>>,
    pub last_lifesign: Arc<Mutex<Instant>>,
    pub last_worldview: Arc<Mutex<UdpMsg>>,
    pub active_elevators: Arc<Mutex<Vec<Cab>>>,
    pub failed_orders: Arc<Mutex<Vec<Order>>>,
    pub sent_messages: Arc<Mutex<Vec<UdpMsg>>>,

}