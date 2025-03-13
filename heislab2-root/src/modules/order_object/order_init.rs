use modules::alias_lib;

// IF CAB THE ORDER IS LOCAL
struct Order {
    floor: u8,
    order_type: u8,
}

impl Order{
    pub fn init(addr: &str, floor: u8, order_type:u8) -> Result<Order> {
        Ok(Self {
            floor: floor,
            order_type: order_type,
        })
    }
}