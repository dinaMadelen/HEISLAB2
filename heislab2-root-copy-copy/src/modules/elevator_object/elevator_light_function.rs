
use super::elevator_init::Elevator; 

impl Elevator{
    pub fn turn_on_queue_lights(&mut self){
        for order in self.queue.clone(){
            self.call_button_light(order.floor, order.order_type, true);
        }
    }
    pub fn turn_off_lights(&mut self){
        for floors in 0..(self.num_floors) {
            for call_types in 0..3 {
                self.call_button_light(floors, call_types, false);
            }
        }
    }
    pub fn turn_off_lights_not_in_queue(&mut self){
        for floors in 0..(self.num_floors) {
            for call_types in 0..3 {
                self.call_button_light(floors, call_types, false);
            }
        }

        for order in self.queue.clone(){
            self.call_button_light(order.floor, order.order_type, true);
        }
    }

    pub fn turn_off_last_order_light(&mut self){
        if let Some(last_order) = self.queue.first() {
            self.call_button_light(last_order.floor, last_order.order_type, false);
        }
    }

    pub fn turn_on_last_order_light(&mut self){
        if let Some(last_order) = self.queue.first() {
            self.call_button_light(last_order.floor, last_order.order_type, true);
        }
    }



}





