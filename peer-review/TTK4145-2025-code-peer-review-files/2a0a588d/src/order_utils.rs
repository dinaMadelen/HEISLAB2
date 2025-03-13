use serde::Serialize;
use crate::elevio::poll::CallButton;
use crate::elevio;


const FLOORS: usize = 4;
const TOP_FLOOR: usize = FLOORS-1;


#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum OrderStatus {
    Empty,
    New,
    Confirmed,
    Delivered,
    NotPossible // In top/bottom floor it is NotPossible to go up/down.
}

#[derive(Debug, Clone)]
pub struct FloorHallButtons {
    pub up:   OrderStatus,
    pub down: OrderStatus,
}

impl Copy for FloorHallButtons {}
impl Copy for OrderStatus {}


impl OrderStatus {
    pub fn increment(&mut self) {
        *self = match *self {
            OrderStatus::Empty => OrderStatus::New,
            OrderStatus::New => OrderStatus::Confirmed,
            OrderStatus::Confirmed => OrderStatus::Delivered,
            OrderStatus::Delivered => OrderStatus::Empty,
            OrderStatus::NotPossible => OrderStatus::NotPossible,
            _ => panic!("Tried to do change increment an invalid state for OrderStatus."),
        }
    }
}


pub fn init_empty_hall_buttons() -> [FloorHallButtons; FLOORS] {
    let mut hall_buttons = [FloorHallButtons{up: OrderStatus::Empty, down: OrderStatus::Empty}; FLOORS];
    hall_buttons[0].down = OrderStatus::NotPossible;
    hall_buttons[TOP_FLOOR].up = OrderStatus::NotPossible;
    hall_buttons
}


pub fn format_2_hall_requests(hall_buttons: [FloorHallButtons; FLOORS]) -> [[bool; 2]; FLOORS] {
    let mut hallRequests = [[false;2];FLOORS];
    let mut i = 0;
    for floor_hall_buttons in hall_buttons {
        hallRequests[i][0] = if floor_hall_buttons.up == OrderStatus::Confirmed {true} else {false};
        hallRequests[i][1] = if floor_hall_buttons.down == OrderStatus::Confirmed {true} else {false};
        i += 1;
    }
    println!("serialized = {}", serde_json::to_string(&hallRequests).unwrap());
    hallRequests
}

pub fn increment_order_at_call_button(call_button: &CallButton, mut orders: [FloorHallButtons; FLOORS]) {
    match call_button.call {
        elevio::elev::HALL_UP => {
            orders[call_button.floor as usize].up.increment();
        }
        elevio::elev::HALL_DOWN => {
            orders[call_button.floor as usize].down.increment();
        }
        other => panic!("func: 'increment_call_button' does not support: {}", other)
    }
}

pub fn confirm_order_at_call_button(call_button: &CallButton, mut orders: [FloorHallButtons; FLOORS]) {
    match call_button.call {
        elevio::elev::HALL_UP => {
            orders[call_button.floor as usize].up = OrderStatus::Confirmed;
        }
        elevio::elev::HALL_DOWN => {
            orders[call_button.floor as usize].down = OrderStatus::Confirmed;
        }
        other => panic!("func: 'confirmed_order' does not support: {}", other)
    }
}

pub fn flag_new_order_at_call_button(call_button: &CallButton,  mut orders: [FloorHallButtons; FLOORS]) {
    match call_button.call {
        elevio::elev::HALL_UP => {
            if orders[call_button.floor as usize].up == OrderStatus::Empty {
                orders[call_button.floor as usize].up.increment();
            }
        }
        elevio::elev::HALL_DOWN => {
            if orders[call_button.floor as usize].up == OrderStatus::Empty {
                orders[call_button.floor as usize].up.increment();
            }
        }
        other => panic!("func: 'confirmed_order' does not support: {}", other)
    }
}