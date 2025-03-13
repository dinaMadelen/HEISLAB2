use std::thread;
use std::time::Duration;
use crossbeam_channel::{self as cbc, select};
use crate::state_utils::{self, fsm_state};
use crate::elevio::{self, elev as e};
use crate::request::requests_choose_direction;


pub fn fsm_main(elevator: e::Elevator, from_state_manager_rx: cbc::Receiver<fsm_state>, from_fsm_tx: cbc::Sender<fsm_state>, order_fulfilled_tx: cbc::Sender<elevio::poll::CallButton>, floor_sensor_rx: cbc::Receiver<u8>, obstruction_rx: cbc::Receiver<bool>, stop_button_rx: cbc::Receiver<bool>) {

    let mut current_state = state_utils::fsm_state {
        behaviour: state_utils::Behaviour::idle,
        floor: elevator.floor_sensor().unwrap(),
        direction: state_utils::Direction::stop,
        requests: [[false; 3]; 4]
    };

    let mut obstruction = false;

    let (door_open_tx, door_open_rx) = cbc::unbounded::<bool>();
    let (door_timer_tx, door_timer_rx) = cbc::unbounded::<bool>();
    {
        thread::spawn(move || door_controller(door_open_rx, door_timer_tx));
    }

    loop {
        select! {
            recv(from_state_manager_rx) -> new_state => {
                current_state = new_state.unwrap();
                (current_state.direction, current_state.behaviour) = requests_choose_direction(current_state);
            },
            recv(floor_sensor_rx) -> floor => {
                current_state.floor = floor.unwrap();
                elevator.floor_indicator(current_state.floor);

                let floor_idx = current_state.floor as usize;
                for (i, req) in current_state.requests[floor_idx].iter_mut().enumerate() {
                    if *req {
                        *req = false;
                        order_fulfilled_tx.send(elevio::poll::CallButton {
                            floor: floor_idx as u8,
                            call: i as u8
                        }).unwrap();
                        current_state.behaviour = state_utils::Behaviour::doorOpen;
                    }
                }

                from_fsm_tx.send(current_state).unwrap();
            },
            recv(door_timer_rx) -> _ => {
                if obstruction {
                    door_open_tx.send(true).unwrap();
                } else {
                    current_state.behaviour = state_utils::Behaviour::idle;
                    from_fsm_tx.send(current_state).unwrap();
                }
            },
            recv(obstruction_rx) -> obst => {
                obstruction = obst.unwrap();
            },
            recv(stop_button_rx) -> _ => {
                current_state.behaviour = state_utils::Behaviour::idle;
                elevator.motor_direction(e::DIRN_STOP);
                from_fsm_tx.send(current_state).unwrap();
            },
            default => {
                match current_state.behaviour {
                    state_utils::Behaviour::idle => {
                        elevator.door_light(false);
                        elevator.motor_direction(e::DIRN_STOP);
                    },
                    state_utils::Behaviour::doorOpen => {
                        elevator.door_light(true);
                        elevator.motor_direction(e::DIRN_STOP);
                        door_open_tx.send(true).unwrap();
                    },
                    state_utils::Behaviour::moving => {
                        elevator.motor_direction(current_state.direction as u8);
                    }
                }
            }
        }
        thread::sleep(Duration::from_millis(15));
    }
}


fn door_controller(door_open_rx: cbc::Receiver<bool>, door_timer_tx: cbc::Sender<bool>) {
    loop {
        if let Ok(true) = door_open_rx.recv() {
            println!("Door opening triggered.");

            let mut timer = cbc::after(Duration::from_secs(3));
            loop {
                select! {
                    recv(timer) -> _ => {
                        println!("3 seconds elapsed. Door will now close.");
                        door_timer_tx.send(true).unwrap();
                        break;
                    },
                    recv(door_open_rx) -> msg => {
                        if let Ok(true) = msg {
                            println!("Received additional door open signal, resetting timer.");
                            timer = cbc::after(Duration::from_secs(3));
                        }
                    }
                }
            }
        }
    }
}