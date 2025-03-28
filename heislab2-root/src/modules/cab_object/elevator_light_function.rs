
use crate::modules::elevator_object::elevator_init::Elevator; 
use super::cab::Cab;
use crate::modules::order_object::order_init::Order;
use crate::modules::elevator_object::alias_lib::{HALL_DOWN, HALL_UP};
use std::thread::*;
use std::time::*;
use crossbeam_channel as cbc;
//use heislab2_root::modules::io::io_init;
//use heislab2_root::modules::master_functions::master::handle_slave_failure;
use std::sync::Arc;
use std::net::{SocketAddr, IpAddr, Ipv4Addr};

use crate::modules::elevator_object::elevator_init::SystemState;
use crate::modules::elevator_object::alias_lib::CAB;




impl Cab{
    pub fn turn_on_hall_lights(&mut self,   elevator: Elevator, order_vec: Vec<Order>){

        for order in order_vec{
            if order.order_type == HALL_UP || order.order_type == HALL_DOWN {
                elevator.call_button_light(order.floor, order.order_type, true);
            } 
        }
    }
    
    pub fn turn_off_lights(&mut self, elevator:Elevator){
        for floors in 0..(self.num_floors) {
            for call_types in 0..3 {
                elevator.call_button_light(floors, call_types, false);
            }
        }
    }
    pub fn lights(&mut self, state: &Arc<SystemState>, elevator:Elevator){
        // Turn off lights for orders that are no longer in the new order vector.
        for floor in 0..self.num_floors {
            for call_type in 0..3 {
                let order = Order::init(floor, call_type);
                let known_elevators_clone = state.known_elevators.lock().unwrap().clone();
                let mut should_light = false;
    
                // For hall orders, check if any known elevator has the order in its queue.
                if order.order_type == HALL_UP || order.order_type == HALL_DOWN {
                    for cab in known_elevators_clone.iter() {
                        if cab.queue.contains(&order) {
                            should_light = true;
                            break;
                        }
                    }
                }
    
                // Also check if the current elevator's queue contains the order.
                if self.queue.contains(&order) {
                    should_light = true;
                }
    
                // Set the light once, based on the aggregated condition.
                elevator.call_button_light(order.floor, order.order_type, should_light);
            }
        }
    }

}





