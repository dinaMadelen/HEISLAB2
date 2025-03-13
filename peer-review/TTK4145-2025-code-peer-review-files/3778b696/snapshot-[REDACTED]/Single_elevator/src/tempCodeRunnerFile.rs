use driver_rust::elevio;
use driver_rust::elevio::elev::Elevator;

use crate::{N_BUTTONS, N_FLOORS};

#[derive(Debug)]
pub enum ElevatorState {
    Idle,
    Moving,
    DoorOpen,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ClearRequestVariant {
    All,    // Alle som venter går inn uansett retning
    InDirn, // Kun de som vil i heisens retning går inn
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Direction {
    Up,
    Down,
    Stop,
}

pub struct ElevatorStateMachine {
    state: ElevatorState,
    current_floor: u8,
    direction: Direction,
    requests: [[bool; N_BUTTONS]; N_FLOORS],
}

pub struct ElevatorConfig {
    clear_request_variant: ClearRequestVariant, // Hvordan heisen håndterer requests
    door_open_duration_s: f64,                 // Hvor lenge dørene skal være åpne
}

impl ElevatorStateMachine {
    fn set_all_lights(&self, elevator: &Elevator) {
        for floor in 0..N_FLOORS {
            for btn in 0..N_BUTTONS {
                elevator.call_button_light(floor as u8, btn as u8, self.requests[floor][btn]);
            }
        }
    }
}

pub fn on_request_button(button: elevio::poll::CallButton, fsm: &mut ElevatorStateMachine, elevator: &Elevator) {
    println!("Knappetrykk: {:?}", button);
    elevator.call_button_light(button.floor, button.call, true);

    match fsm.state {
        ElevatorState::Idle => {
            println!("Heisen starter å kjøre til {}", button.floor);
            fsm.state = ElevatorState::Moving;
            if button.floor > fsm.current_floor {
                elevator.motor_direction(elevio::elev::DIRN_UP);
            } else {
                elevator.motor_direction(elevio::elev::DIRN_DOWN);
            }
        },
        ElevatorState::Moving => {
            println!("Heisen er allerede i bevegelse.");
        },
        ElevatorState::DoorOpen => {
            println!("Dørene er åpne, venter...");
        },
    }
}



        


    


