use serde::{Serialize, Deserialize};


// IF CAB THE ORDER IS LOCAL
#[derive(Clone, Debug,PartialEq, Serialize, Deserialize)]
pub struct Order {
    pub floor: u8,
    pub order_type: u8,
}

impl Order{
    pub fn init(floor: u8, order_type:u8) -> Order {
        Self {
            floor: floor,
            order_type: order_type,
        }
    }

}