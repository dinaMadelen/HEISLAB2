use crate::elevio::elev::{HALL_UP, HALL_DOWN, CAB};

use crate::state_utils::{Direction, Behaviour, fsm_state};

const N_FLOORS: u8 = 4;
const N_BUTTONS: u8 = 3;

fn requests_above(state: fsm_state) -> bool {
    for f in state.floor+1..N_FLOORS {
        for btn in 0..N_BUTTONS {
            if state.requests[f as usize][btn as usize] {
                return true;
            }
        }
    }
    return false;
}

fn requests_below(state: fsm_state) -> bool {
    for f in 0..state.floor{
        for btn in 0..N_BUTTONS {
            if(state.requests[f as usize][btn as usize]){
                return true;
            }
        }
    }
    return false;
}

fn requests_here(state: fsm_state) -> bool {
    for btn in 0..N_BUTTONS {
        if state.requests[state.floor as usize][btn as usize] {
            return true;
        }
    }
    return false;
}

pub fn requests_choose_direction(state: fsm_state) -> (Direction, Behaviour) {
    match state.direction {
        Direction::up => {
            if requests_above(state.clone()) { return (Direction::up, Behaviour::moving); }
            if  requests_here(state.clone()) { return (Direction::down, Behaviour::doorOpen); }
            if requests_below(state.clone()) { return (Direction::down, Behaviour::moving); }
            return (Direction::stop, Behaviour::idle);
        }
        Direction::down => {
            if requests_below(state.clone()) { return (Direction::down, Behaviour::moving); }
            if  requests_here(state.clone()) { return (Direction::up, Behaviour::doorOpen); }
            if requests_above(state.clone()) { return (Direction::up, Behaviour::moving); }
            return (Direction::stop, Behaviour::idle);
        }
        Direction::stop => { // there should only be one request in the Stop case. Checking up or down first is arbitrary.
            if  requests_here(state.clone()) { return (Direction::stop, Behaviour::doorOpen); }
            if requests_above(state.clone()) { return (Direction::up, Behaviour::moving); }
            if requests_below(state.clone()) { return (Direction::down, Behaviour::moving); }
            return (Direction::stop, Behaviour::idle);
        }
        _ => {return (Direction::stop, Behaviour::idle); }
    }
}



fn requests_shouldStop(state: fsm_state) -> bool {
    match state.direction {
    Direction::down => {
        return
        state.requests[state.floor as usize][HALL_DOWN as usize] ||
        state.requests[state.floor as usize][CAB as usize]      ||
        !requests_below(state);
    }
    Direction::up => {
        return
        state.requests[state.floor as usize][HALL_UP as usize]   ||
        state.requests[state.floor as usize][CAB as usize]      ||
        !requests_above(state);
    } 
    _ => { return true; }
    }
}


enum ClearRequestVariant {
    // Assume everyone waiting for the elevator gets on the elevator, even if 
    // they will be traveling in the "wrong" direction for a while
    CV_All,
    
    // Assume that only those that want to travel in the current direction 
    // enter the elevator, and keep waiting outside otherwise
    CV_InDirn,
}


pub fn requests_shouldClearImmediately(state: fsm_state, clearRequestVariant: ClearRequestVariant, btn_floor: u8, btn_type: u8) -> bool {
    match clearRequestVariant {
        ClearRequestVariant::CV_All => { return state.floor == btn_floor; }
        ClearRequestVariant::CV_InDirn => {
            return 
            state.floor == btn_floor && 
            (
                (state.direction == Direction::up   && btn_type == HALL_UP)    ||
                (state.direction == Direction::down && btn_type == HALL_DOWN)  ||
                state.direction == Direction::stop ||
                btn_type == CAB
            );  
        }
        _ => { return false; }
    }
}


pub fn requests_clearAtCurrentFloor(mut state: fsm_state, clearRequestVariant: ClearRequestVariant) -> fsm_state {
    match clearRequestVariant {
        ClearRequestVariant::CV_All => {
            for btn in 0..N_BUTTONS {
                state.requests[state.floor as usize][btn as usize] = false;
            }
        },
        ClearRequestVariant::CV_InDirn => {
            state.requests[state.floor as usize][CAB as usize] = false;
            match state.direction {
                Direction::up => {
                    if(!requests_above(state.clone()) && !state.requests[state.floor as usize][HALL_UP as usize]){
                        state.requests[state.floor as usize][HALL_DOWN as usize] = false;
                    }
                    state.requests[state.floor as usize][HALL_UP as usize] = false;
                },
                Direction::down => {
                    if(!requests_below(state.clone()) && !state.requests[state.floor as usize][HALL_DOWN as usize]){
                        state.requests[state.floor as usize][HALL_UP as usize] = false;
                    }
                    state.requests[state.floor as usize][HALL_DOWN as usize] = false;
                },
                Direction::stop => {},
                _ => {
                    state.requests[state.floor as usize][HALL_UP as usize] = false;
                    state.requests[state.floor as usize][HALL_DOWN as usize] = false;
                }
            }
        }
        _ => {}
    }
    
    return state;
}
