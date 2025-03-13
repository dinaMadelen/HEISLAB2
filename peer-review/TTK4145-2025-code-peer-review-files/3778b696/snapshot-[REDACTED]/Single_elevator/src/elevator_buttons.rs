use driver_rust::elevio;
use driver_rust::elevio::elev::Elevator;



pub fn button_type_to_string(button_type: u8) -> &'static str {
    match button_type {
        0 => "HALL_UP",
        1 => "HALL_DOWN",
            2 => "CAB",
            _ => "UNKNOWN",
    }
    
    }