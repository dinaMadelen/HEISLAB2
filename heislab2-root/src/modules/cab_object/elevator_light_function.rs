
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

use crate::modules::master_functions::master::Role;
use crate::modules::elevator_object::elevator_init::SystemState;
use crate::modules::elevator_object::alias_lib::{CAB};


use std::thread::sleep;
use std::time::Duration; //https://doc.rust-lang.org/std/time/struct.Instant.html
use std::env; // Used for reboot function
use std::process::{Command, exit}; //Used for reboot function



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
        for floors in 0..(self.num_floors) {
            for call_types in 0..3 {
                let order = Order::init(floors, call_types);
                let known_elevators_clone = state.known_elevators.lock().unwrap().clone();
                
                for cab in known_elevators_clone.iter() {
                    // If the order is a hall order, check if it's in the elevator's queue.
                    if (order.order_type == HALL_UP || order.order_type == HALL_DOWN) 
                        && cab.queue.contains(&order)
                    {
                        elevator.call_button_light(order.floor, order.order_type, true);
                    }
                    // Alternatively, if you also want to check the current elevator's queue (self.queue)
                    // and set the light if the order exists there, you could do:
                    if self.queue.contains(&order) {
                        elevator.call_button_light(order.floor, order.order_type, true);
                    }else{

                    elevator.call_button_light(order.floor, order.order_type, false);
                    }
                }
            }
        }
    }

}





