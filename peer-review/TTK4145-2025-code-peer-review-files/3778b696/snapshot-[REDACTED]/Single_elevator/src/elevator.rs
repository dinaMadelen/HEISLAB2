use driver_rust::elevio::elev as e;


pub const N_BUTTONS :usize = 3;
pub const N_SHARED_BUTTONS : usize=N_BUTTONS-1; //number of buttons which are equal between elevators.
pub const N_FLOORS : usize = 4;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Up,
    Down,
    Stop,
}

impl Direction {
    pub fn to_driver_value(&self) -> u8 {
        match self {
            Direction::Up => e::DIRN_UP,
            Direction::Down => e::DIRN_DOWN,
            Direction::Stop => e::DIRN_STOP,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ElevatorBehaviour {
    Idle,
    DoorOpen,
    Moving,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq,serde::Serialize,serde::Deserialize)]
pub enum ButtonType {
    HallUp,
    HallDown,
    Cab,
}

impl ButtonType {
    pub fn from_u8(value: u8) -> Self {
        match value {
            0 => ButtonType::HallUp,
            1 => ButtonType::HallDown,
            2 => ButtonType::Cab,
            _ => ButtonType::Cab, // Default fallback (skal ikke skje)
        }
    }

    pub fn to_u8(&self) -> u8 {
        match self {
            ButtonType::HallUp => 0,   //noen trykker på opp knappen i hallen
            ButtonType::HallDown => 1, //noen trykker på ned knappen i hallen
            ButtonType::Cab => 2,
        }
    }
}

pub struct Elevator {
    pub floor: i32,
    pub dirn: Direction,
    pub behaviour: ElevatorBehaviour,
}

impl Elevator {
    pub fn uninitialized() -> Self {
        Elevator {
            floor: -1,
            dirn: Direction::Stop,
            behaviour: ElevatorBehaviour::Idle,
        }
    }

    pub fn set_motor_direction(&self, hw_elevator: &e::Elevator) {
        hw_elevator.motor_direction(self.dirn.to_driver_value());
    }
}
