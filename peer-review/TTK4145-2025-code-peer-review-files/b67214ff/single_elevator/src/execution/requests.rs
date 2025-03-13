use crate::execution::elevator::{self, Button, Dirn, Elevator, ElevatorBehaviour, N_BUTTONS, N_FLOORS};


pub struct DirnBehaviourPair {
    pub dirn : Dirn,
    pub behaviour : ElevatorBehaviour
}

pub fn requests_above(e : &Elevator) -> bool {
    for f in (e.floor.expect("Floor is None")+1) as usize..N_FLOORS {
        for btn in 0..N_BUTTONS {
            if e.requests[f][btn] {
                return true;
            }
        }
    }
    return false;
}

pub fn requests_below(e : &Elevator) -> bool {
    for f in 0..e.floor.expect("Floor is None") {
        for btn in 0..N_BUTTONS {
            if e.requests[f as usize][btn] {
                return true;
            }
        }
    }
    return false;
}

pub fn requests_here(e : &Elevator) -> bool {
    for btn in 0..N_BUTTONS {
        if e.requests[e.floor.expect("Floor is None") as usize][btn] {
            return true;
        }
    }
    return false;
}

pub fn requests_choose_direction(e : &Elevator) -> DirnBehaviourPair {
    match e.dirn {
        Dirn::Up => {
            return if requests_above(&e) {DirnBehaviourPair{dirn : Dirn::Up , behaviour : ElevatorBehaviour::Moving}}
                else if requests_here(&e) {DirnBehaviourPair{dirn : Dirn::Down , behaviour : ElevatorBehaviour::DoorOpen}}
                else if requests_below(&e) {DirnBehaviourPair{dirn : Dirn::Down , behaviour : ElevatorBehaviour::Moving}}
                else { DirnBehaviourPair{dirn : Dirn::Stop , behaviour : ElevatorBehaviour::Idle}}
        }
        Dirn::Down => {
            return if requests_below(&e) {DirnBehaviourPair{dirn : Dirn::Down , behaviour : ElevatorBehaviour::Moving}}
                else if requests_here(&e) {DirnBehaviourPair{dirn : Dirn::Up , behaviour : ElevatorBehaviour::DoorOpen}}
                else if requests_above(&e) {DirnBehaviourPair{dirn : Dirn::Up , behaviour : ElevatorBehaviour::Moving}}
                else { DirnBehaviourPair{dirn : Dirn::Stop , behaviour : ElevatorBehaviour::Idle}}

        }
        Dirn::Stop => {
            return if requests_here(&e) {DirnBehaviourPair{dirn : Dirn::Stop , behaviour : ElevatorBehaviour::DoorOpen}}
                else if requests_above(&e) {DirnBehaviourPair{dirn : Dirn::Up , behaviour : ElevatorBehaviour::Moving}}
                else if requests_below(&e) {DirnBehaviourPair{dirn : Dirn::Down , behaviour : ElevatorBehaviour::Moving}}
                else { DirnBehaviourPair{dirn : Dirn::Stop , behaviour : ElevatorBehaviour::Idle}}

        }
        _ => {
            eprintln!("Request_choose_direction function in request.rs got invalid direction from elevator");
            return DirnBehaviourPair{dirn : Dirn::Stop , behaviour : ElevatorBehaviour::Idle}
        }
    }
}


pub fn requests_should_stop(e : &Elevator) -> bool {
    match e.dirn {
        Dirn::Down => {
            return 
                e.requests[e.floor.expect("Floor is None") as usize][Button::HallDown as usize] ||
                e.requests[e.floor.expect("Floor is None") as usize][Button::Cab as usize] ||
                !requests_below(&e);
        }
        Dirn::Up => {
            return 
                e.requests[e.floor.expect("Floor is None")as usize][Button::HallUp as usize] ||
                e.requests[e.floor.expect("Floor is None")as usize][Button::Cab as usize]  ||
                !requests_above(&e);

        }
        Dirn::Stop => {
            return true;
        }
        _ => {
            eprintln!("Request_should_stop function in request.rs got invalid direction from elevator");
            return true;
        }

    }
}


pub fn requests_should_clear_immediately_matrix(e : &mut Elevator ) -> bool { //Not great that this now changes the elevator requests. Might be a problem
    let mut should_clear = false;

    for f in 0..N_FLOORS {
        for btn_type in  0..N_BUTTONS {
            if (e.floor.expect("Floor is None") == f as u8) && (
                (e.dirn == Dirn::Up && btn_type == Button::HallUp as usize) ||
                (e.dirn == Dirn::Down && btn_type == Button::HallDown as usize) ||
                e.dirn == Dirn::Stop ||
                btn_type == Button::Cab as usize
            ) {
                should_clear = true;
                e.requests[f][btn_type] = false;
            }
        }  
    }
    return should_clear
} 




pub fn requests_clear_at_current_floor(elevator: &mut elevator::Elevator) {
    
    
    let floor = elevator.floor.expect("Floor is None") as usize;
    let dirn = elevator.dirn;
    let requests_above_exist = requests_above(elevator);
    let requests_below_exist = requests_below(elevator);
    
    elevator.requests[floor as usize][Button::Cab as usize] = false; //Should this be sent to the controller?
    
    match dirn {
        Dirn::Up => {
            if !requests_above_exist && !elevator.requests[floor][Button::HallUp as usize] {
                elevator.requests[floor][Button::HallDown as usize] = false;
            }
            elevator.requests[floor][Button::HallUp as usize] = false;
        }
        Dirn::Down => {
            if !requests_below_exist && !elevator.requests[floor][Button::HallDown as usize] {
                elevator.requests[floor][Button::HallUp as usize] = false;
            }
            elevator.requests[floor][Button::HallDown as usize] = false;
            
        }
        Dirn::Stop => {
            elevator.requests[floor][Button::HallUp as usize] = false;
            elevator.requests[floor][Button::HallDown as usize] = false;
        }
    }
}
