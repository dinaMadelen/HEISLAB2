
use crate::modules::elevator_object::elevator_init::Elevator; 
use super::cab::Cab;
use crate::modules::order_object::order_init::Order;
use crate::modules::elevator_object::alias_lib::{HALL_DOWN, HALL_UP};

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
    
    pub fn lights(&mut self, order_vec: Vec<Order>,  elevator:Elevator){
        // Turn off lights for orders that are no longer in the new order vector.
        for floors in 0..(self.num_floors) {
            for call_types in 0..3 {
                let order = Order::init(floors, call_types);
                if order_vec.contains(&order) {
                    
                    if order.order_type == HALL_UP || order.order_type == HALL_DOWN {
                        elevator.call_button_light(order.floor, order.order_type, true);
                    } 
                    if self.queue.contains(&order){
                        elevator.call_button_light(order.floor, order.order_type, true);
                    }   
                }else{
                    elevator.call_button_light(order.floor, order.order_type, false);
                }
                if self.queue.is_empty(){
                    elevator.call_button_light(floors, call_types, false);
                }
            }
        }
    }



}





