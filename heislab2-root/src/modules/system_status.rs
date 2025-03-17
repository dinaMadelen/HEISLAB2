use crate::modules::order_object::order_init::Order;
use crate::modules::elevator_object::elevator_init::Elevator;
use crate::modules::udp::{UdpMsg};

use std::sync::{Arc, Mutex};

pub struct SystemState {
    pub ME : Elevator,
    pub master: Elevator,
    pub active_elevators: Arc<Mutex<Vec<Elevator>>>,
    pub failed_orders: Arc<Mutex<Vec<Order>>>,
    pub sent_messages: Arc<Mutex<Vec<UdpMsg>>>,

}