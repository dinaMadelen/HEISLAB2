
use std::time::SystemTime;

use crossbeam_channel::{Receiver, Sender};

use crate::elevio::poll::CallButton;
use crate::elevio::poll;
use crate::execution::requests::{self, DirnBehaviourPair};
use crate::execution::timer;

use crate::execution::elevator::{self, ElevatorBehaviour, Button, Dirn, N_BUTTONS, N_FLOORS};
use crate::interface::HallRequestMatrix;
use crate::logic::controller::ElevatorArgument;

pub struct FSM {
    elevator : elevator::Elevator,
    door_timer : timer::Timer, 

    door_obstructed: bool,

    //For communication with driver
    motor_direction_tx: Sender<u8>,
    call_button_light_tx: Sender<(u8,u8,bool)>,
    floor_indicator_tx: Sender<u8>,
    door_light_tx: Sender<bool>,

    call_button_rx: Receiver<poll::CallButton>,
    floor_sensor_rx: Receiver<u8>,
    obstruction_rx: Receiver<bool>,

    //For communication with controller/interface
    elevator_number:u8, 
    order_state_tx: Sender<(CallButton, bool)>, 
    elevator_argument_tx: Sender<(u8,ElevatorArgument)>, 
    hall_request_rx: Receiver<(u8, HallRequestMatrix)>,
    obstruction_switch_tx: Sender<bool>,

    elev_argument : ElevatorArgument
}


impl FSM {

pub fn init(elevator : elevator::Elevator, 

        motor_direction_tx: Sender<u8>, 
        call_button_light_tx: Sender<(u8,u8,bool)>, 
        floor_indicator_tx: Sender<u8>, 
        door_light_tx: Sender<bool>, 
        call_button_rx: Receiver<poll::CallButton>, 
        floor_sensor_rx: Receiver<u8>, 
        obstruction_rx: Receiver<bool>,    
        
        elevator_number:u8, 
        order_state_tx: Sender<(CallButton, bool)>, 
        elevator_argument_tx: Sender<(u8,ElevatorArgument)>, 
        hall_request_rx: Receiver<(u8, HallRequestMatrix)>,
        obstruction_switch_tx: Sender<bool>,
        ) -> Self {
        Self {
            elevator,
            door_timer: timer::Timer::init(
                SystemTime::now(),
                false
            ),
            door_obstructed : false,

            motor_direction_tx,
            call_button_light_tx,
            floor_indicator_tx,
            door_light_tx,

            call_button_rx,
            floor_sensor_rx,
            obstruction_rx,

            elevator_number, 
            order_state_tx, 
            elevator_argument_tx, 
            hall_request_rx,
            obstruction_switch_tx,

            elev_argument : ElevatorArgument {behaviour: ElevatorBehaviour::Idle, floor : 0, direction : Dirn::Stop, cab_requests : [false; N_FLOORS]}
        }
}

fn set_all_lights(&self) { 
    for floor in 0..N_FLOORS {
        for btn in 0..N_BUTTONS {
            self.call_button_light_tx.send((floor as u8, btn as u8, self.elevator.requests[floor][btn])).expect("call_button_light_tx");
        }
    }
}


fn fsm_on_init_between_floors(&mut self) {

    self.motor_direction_tx.send(Dirn::Down as u8).expect("motor_direction_tx"); 
    self.elevator.dirn = Dirn::Stop;
    self.elevator.behaviour = ElevatorBehaviour::Moving;

}


fn fsm_on_request_matrix(&mut self) {

    match self.elevator.behaviour  {

        ElevatorBehaviour::DoorOpen => {
            if requests::requests_should_clear_immediately_matrix(&mut self.elevator) {
              self.door_timer.timer_start(self.elevator.door_open_duration_s);  
            } 
        }

        ElevatorBehaviour::Moving => {}
        ElevatorBehaviour::Idle => {
            let pair : DirnBehaviourPair  = requests::requests_choose_direction(&self.elevator);
            self.elevator.dirn = pair.dirn;
            self.elevator.behaviour = pair.behaviour;

            match self.elevator.behaviour {

                ElevatorBehaviour::DoorOpen => {
                    self.door_light_tx.send(true).expect("door_light_tx");
                    self.door_timer.timer_start(self.elevator.door_open_duration_s);
                    requests::requests_clear_at_current_floor(&mut self.elevator);
                }

                ElevatorBehaviour::Moving => {
                    self.motor_direction_tx.send(self.elevator.dirn as u8).expect("motor_direction_tx");
                }

                ElevatorBehaviour::Idle => {
                }
            }
        }
    }
     self.set_all_lights();

}

   

fn fsm_on_floor_arrival(&mut self, new_floor :Option<u8> ) {
    self.elevator.floor = new_floor;
    match self.elevator.floor {
        Some(floor) => {
            self.floor_indicator_tx.send(floor).expect("floor_indicator_tx");
        },
        None => (),
    }


    match self.elevator.behaviour {
        ElevatorBehaviour::Moving => {
            if requests::requests_should_stop(&self.elevator) {
                self.motor_direction_tx.send(Dirn::Stop as u8).expect("motor_direction_tx");
                self.door_light_tx.send(true).expect("door_light_tx");
                requests::requests_clear_at_current_floor(&mut self.elevator);
                self.door_timer.timer_start(self.elevator.door_open_duration_s);  
                self.set_all_lights();
                self.elevator.behaviour = ElevatorBehaviour::DoorOpen;
            }
        }
        _ => {
        }
    }

}


fn fsm_on_door_timeout(&mut self) {

    match self.elevator.behaviour  {

        ElevatorBehaviour::DoorOpen => {

            if self.door_obstructed {
                self.door_timer.timer_start(self.elevator.door_open_duration_s); 
                return;
            }

            let pair : DirnBehaviourPair = requests::requests_choose_direction(&mut self.elevator);
            self.elevator.dirn = pair.dirn;
            self.elevator.behaviour = pair.behaviour;

            match self.elevator.behaviour {

                ElevatorBehaviour::DoorOpen => {
                    self.door_timer.timer_start(self.elevator.door_open_duration_s);
                    requests::requests_clear_at_current_floor(&mut self.elevator);               
                }
                
                ElevatorBehaviour::Moving => {
                    self.door_light_tx.send(false).expect("door_light_tx");
                    self.motor_direction_tx.send(self.elevator.dirn as u8).expect("motor_direction_tx");
                }

                ElevatorBehaviour::Idle => {
                    self.door_light_tx.send(false).expect("door_light_tx");
                    self.motor_direction_tx.send(self.elevator.dirn as u8).expect("motor_direction_tx");
                }
                
            }
            self.set_all_lights();
        }
        _ => {
        }
    }
}

pub fn update_elevator_arguments(&mut self) -> bool {
    let mut elevator_changes = false;

    if self.elev_argument.direction != self.elevator.dirn{
        self.elev_argument.direction = self.elevator.dirn;
        elevator_changes = true;
    }

    if self.elev_argument.behaviour != self.elevator.behaviour{
        self.elev_argument.behaviour = self.elevator.behaviour;
        elevator_changes = true;
    }

    match self.elevator.floor {
        Some(f) => {
            if self.elev_argument.floor != f as usize{
                self.elev_argument.floor = f as usize;
                elevator_changes = true;
            }
        },
        None => (),
    }

    for f in 0..N_FLOORS {
        if self.elev_argument.cab_requests[f] != self.elevator.requests[f][2] {
            self.elev_argument.cab_requests[f] = self.elevator.requests[f][2];
            elevator_changes = true;
        }
    }
    elevator_changes
}




pub fn run_fsm(&mut self) {


    match self.floor_sensor_rx.try_recv() {
        Ok(_) => (),
        Err(_) =>{
                self.fsm_on_init_between_floors();
            }
    }


    println!("FSM initialized");

    loop {

        match self.call_button_rx.try_recv() {
            Ok(call_button) => {
                if call_button.call == Button::Cab as u8 {
                    self.elevator.requests[call_button.floor as usize][call_button.call as usize] = true;
                    self.fsm_on_request_matrix();
                }
                else {
                    self.order_state_tx.send((call_button, true)).expect("order_state_tx");
                }
            },
            Err(_error) => (),
        }

        match self.floor_sensor_rx.try_recv() {
            Ok(floor) => {
                self.fsm_on_floor_arrival(Some(floor));
            },
            Err(_error) => (),
        }

        match self.obstruction_rx.try_recv() {
            Ok (on) => {
                self.door_obstructed = on;
                self.obstruction_switch_tx.send(on).expect("obstruction_switch_tx");
            },
            Err(_error) => (),
        }

        { // Timer
            if self.door_timer.timer_timed_out() {
                self.door_timer.timer_stop();
                self.fsm_on_door_timeout();
            }

        }

        //Communication with Controller:

        match self.hall_request_rx.try_recv() {
            Ok ((elev_num, hall_requests)) => {
                let mut new_requests = false;
                if elev_num == self.elevator_number{
                    for f in 0..N_FLOORS {
                        for btn_type in  0..N_BUTTONS-1 {

                            if self.elevator.requests[f][btn_type] != hall_requests[f][btn_type] {
                                self.elevator.requests[f][btn_type] = hall_requests[f][btn_type];
                                new_requests = true;
                            }
                        }
                    }
                    if new_requests {
                        self.fsm_on_request_matrix();
                    }
                }
            },
            Err(_error) => (),
        }

        if self.update_elevator_arguments() {
            self.elevator_argument_tx.send((self.elevator_number, self.elev_argument)).expect("elevator_argument_tx");
        };
    }
}


}