#![allow(dead_code)]
#![warn(unused_variables)]
#[allow(unused_imports)]

use crate::modules::elevator_object::*;
use crate::modules::order_object::order_init::Order;

use crate::modules::elevator_object::elevator_init::Elevator; 

impl Elevator{
    pub fn add_to_queue(&mut self, order:Order) {
        if !self.queue.contains(&order) {
            self.queue.push(order);
            self.sort_queue();
        }
        else{
            self.print_status();
        }
    }
    
    //DENNE MÅ ENDRES
    pub fn sort_queue(&self) -> Vec<Order> {
        //todo!("MAKE SORT QUEUE ACTUALLY SORT QUEUE");

        return self.queue.clone();

    }

}