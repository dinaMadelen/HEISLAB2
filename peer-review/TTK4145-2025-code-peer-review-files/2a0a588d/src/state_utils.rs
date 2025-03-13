use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use std::fmt;
use duplicate::duplicate_item;

use crate::elevio::elev::{HALL_UP, HALL_DOWN, CAB};

const FLOORS: usize = 4;

#[allow(non_camel_case_types)]
#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub enum Behaviour {
     idle,
     moving,
     doorOpen
}

#[allow(non_camel_case_types)]
#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub enum Direction {
    up,
    down,
    stop
}

impl PartialEq for Direction {
    fn eq(&self, other: &Self) -> bool {
        self == other
    }
}

#[duplicate_item(name; [Behaviour]; [Direction])]
impl fmt::Display for name {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
        // or, alternatively:
        // fmt::Debug::fmt(self, f)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct State {
    pub behaviour: Behaviour,
    pub floor: u8,
    pub direction: Direction,
    pub cabRequests: [bool;FLOORS],
}

impl State {
    pub fn init() -> State {
        State {
            behaviour: Behaviour::idle,
            floor: 0,
            direction: Direction::stop,
            cabRequests: [false;FLOORS],
        }
    }
    pub fn update(&mut self, fsm_state: fsm_state) {
        self.behaviour = fsm_state.behaviour;
        self.floor = fsm_state.floor;
        self.direction = fsm_state.direction;
        // Ignore cabRequest field from FSM
    }
}

#[derive(Clone, Copy)]
pub struct fsm_state {
    pub behaviour: Behaviour,
    pub floor: u8,
    pub direction: Direction,
    pub requests: [[bool;3];FLOORS],
}

pub fn create_fsm_state(state: &State, hall_orders: [[bool;2];FLOORS]) -> fsm_state {
    let mut requests: [[bool;3];FLOORS] = [[false;3];FLOORS];
    for f in 0..FLOORS {
        requests[f as usize][HALL_UP as usize] = hall_orders[f as usize][0]; // up
        requests[f as usize][HALL_DOWN as usize] = hall_orders[f as usize][1]; // down
        requests[f as usize][CAB as usize] = state.cabRequests[f as usize]; // down
    }
    fsm_state {
        behaviour: state.behaviour.clone(),
        floor:     state.floor.clone(),
        direction: state.direction.clone(),
        requests: requests
    }
}

fn format_2_elevator_state(ip: String, behaviour: Behaviour, floor: u8, direction: Direction, cab_requests: [bool; FLOORS]) -> serde_json::Value {
    serde_json::json!({
        ip: {
            "behaviour": behaviour.to_string(),
            "floor": floor,
            "direction" : direction.to_string(),
            "cabRequests" : cab_requests
        }
    })
}


pub fn elevator_is_alone(states: &HashMap::<String, State>) -> bool {
    if states.len() == 1 {
        return true
    }
    false
}