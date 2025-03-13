use std::time::Duration;

pub const N_FLOORS :usize = 4;
pub const N_BUTTONS :usize = 3;


#[derive(Copy, Clone, Debug, PartialEq,serde::Deserialize,serde::Serialize)]
#[repr(u8)]
pub enum Dirn { 
    #[serde(rename = "down")]
    Down = u8::MAX,
    #[serde(rename = "stop")]
    Stop = 0,
    #[serde(rename = "up")]
    Up = 1
}

#[derive(Copy, Clone, Debug, PartialEq, num_derive::FromPrimitive)]
#[repr(u8)]
pub enum Button { 
    HallUp,
    HallDown,
    Cab
}


#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq,serde::Deserialize,serde::Serialize)]
pub enum ElevatorBehaviour {
    #[serde(rename = "idle")]
    Idle,
    #[serde(rename = "doorOpen")]
    DoorOpen,
    #[serde(rename = "moving")]
    Moving
}

#[derive(Clone, Copy)]
pub struct Elevator {
    pub floor : Option<u8>, 
    pub dirn : Dirn,
    pub requests : [[bool; N_BUTTONS];  N_FLOORS],
    pub behaviour : ElevatorBehaviour,
    pub door_open_duration_s : Duration
}


impl Elevator {
pub fn elevator_init() -> Elevator {
    Elevator{
        floor : None,
        dirn : Dirn::Stop,
        behaviour : ElevatorBehaviour::Idle,
        requests: [[false; N_BUTTONS];N_FLOORS],
        door_open_duration_s : Duration::from_secs(3) // Door open timer is set here
    }
}
}
