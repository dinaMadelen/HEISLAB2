
use crate::modules::elevator_object::elevator_init::Elevator; 
use super::cab::Cab;
use crate::modules::order_object::order_init::Order;


impl Cab{
    pub fn turn_on_queue_lights(&mut self, elevator:Elevator){
        for order in self.queue.clone(){
            elevator.call_button_light(order.floor, order.order_type, true);
        }
    }
    
    pub fn turn_off_lights(&mut self, elevator:Elevator){
        for floors in 0..(self.num_floors) {
            for call_types in 0..3 {
                elevator.call_button_light(floors, call_types, false);
            }
        }
    }
    pub fn turn_off_differing_lights(&mut self, elevator:Elevator, order_vec: Vec<Order>){
        // Turn off lights for orders that are no longer in the new order vector.
        for order in &self.queue {
            if !order_vec.contains(order) {
                elevator.call_button_light(order.floor, order.order_type, false);
            }
        }
    }
    pub fn turn_off_lights_not_in_queue(&mut self, elevator:Elevator){
        for floors in 0..(self.num_floors) {
            for call_types in 0..3 {
                elevator.call_button_light(floors, call_types, false);
            }
        }
        for order in self.queue.clone(){
            elevator.call_button_light(order.floor, order.order_type, true);
        }
    }

    pub fn turn_off_last_order_light(&mut self, elevator:Elevator){
        if let Some(last_order) = self.queue.first() {
            elevator.call_button_light(last_order.floor, last_order.order_type, false);
        }
    }

    pub fn turn_on_last_order_light(&mut self, elevator:Elevator){
        if let Some(last_order) = self.queue.first() {
            elevator.call_button_light(last_order.floor, last_order.order_type, true);
        }
    }



}





