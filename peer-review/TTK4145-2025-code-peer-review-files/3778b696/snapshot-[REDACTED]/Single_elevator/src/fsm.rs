use crate::elevator::{ButtonType, Direction, Elevator, ElevatorBehaviour};
use crate::requests::{ Requests, N_FLOORS};
use crossbeam_channel::Receiver;
use driver_rust::elevio::elev::{self as e, DIRN_DOWN};
use std::thread;
use std::time::Duration;


static mut REQUESTS: Option<Requests> = None;

pub fn fsm_handle_new_order(
    elevator: &mut Elevator,
    hw_elevator: &e::Elevator,
    btn_floor: i32,
    btn_type: i32,
) {
    let btn_type_enum = ButtonType::from_u8(btn_type as u8);

    println!(
        "Request button pressed: Floor {}, Type {:?}",
        btn_floor, btn_type_enum
    );
    hw_elevator.call_button_light(btn_floor as u8, btn_type_enum.to_u8(), true);

    unsafe {
        if REQUESTS.is_none() {
            REQUESTS = Some(Requests::new());
        }
        REQUESTS
            .as_mut()
            .unwrap()
            .add_request(btn_floor, btn_type_enum, elevator);
    }

   if elevator.behaviour == ElevatorBehaviour::Idle {
        fsm_find_next_target(elevator, hw_elevator);
    }

}


 

pub fn fsm_on_floor_arrival(elevator: &mut Elevator, hw_elevator: &e::Elevator, new_floor: i32) {
    elevator.floor = new_floor;
    hw_elevator.floor_indicator(new_floor as u8);

    unsafe {
        if let Some(ref mut requests) = REQUESTS {
            if requests.should_stop(elevator) {
                println!("Stopping at floor {}", new_floor);
                hw_elevator.motor_direction(e::DIRN_STOP);

                // Slukk knappelys for etasjen
                //hw_elevator.call_button_light(new_floor as u8, ButtonType::Cab.to_u8(), false);
                //hw_elevator.call_button_light(new_floor as u8, ButtonType::HallUp.to_u8(), false);
                //hw_elevator.call_button_light(new_floor as u8, ButtonType::HallDown.to_u8(), false);
                println!("direction {:?}", elevator.dirn);

                fsm_open_and_close_door(elevator, hw_elevator);

                requests.clear_request(new_floor);

                fsm_find_next_target(elevator, hw_elevator);

                
                // turn off lights
                println!("direction 2 {:?}", elevator.dirn);
                if elevator.dirn == Direction::Up{
                    hw_elevator.call_button_light(new_floor as u8, ButtonType::HallUp.to_u8(), false);
                    hw_elevator.call_button_light(new_floor as u8, ButtonType::Cab.to_u8(), false);
                } else{
                    hw_elevator.call_button_light(new_floor as u8, ButtonType::HallDown.to_u8(), false);
                    hw_elevator.call_button_light(new_floor as u8, ButtonType::Cab.to_u8(), false);
                }




            }
        }
    }
}



pub fn fsm_find_next_target(elevator: &mut Elevator, hw_elevator: &e::Elevator) {

   
    unsafe {
        if let Some(requests) = REQUESTS.as_ref() {
            if let Some(next_floor) = requests.choose_next_floor() {
                if next_floor > elevator.floor {
                    elevator.dirn = Direction::Up;
                } else if next_floor < elevator.floor {
                    elevator.dirn = Direction::Down;
                } else {
                    elevator.dirn = Direction::Stop;
                }
            
                if elevator.dirn == Direction::Stop {
                    println!("Already at floor {}", elevator.floor);
                    fsm_open_and_close_door(elevator, hw_elevator);
                    unsafe {
                        if let Some(ref mut requests) = REQUESTS {
                            requests.clear_request(elevator.floor);
                        }
                    }
                }else {
                    elevator.behaviour = ElevatorBehaviour::Moving;
                println!("Moving towards floor {}", next_floor);
                elevator.set_motor_direction(hw_elevator);
                }
                
            } else {
                println!("No more requests. Elevator is idle.");
                elevator.dirn = Direction::Stop;
                elevator.behaviour = ElevatorBehaviour::Idle;
            }
        }
    }
    
    unsafe {
        if let Some(requests) = REQUESTS.as_ref() {
            requests.print_queue();
        }
    }
    }



pub fn fsm_open_and_close_door(elevator: &mut Elevator, hw_elevator: &e::Elevator) {
    println!("Opening door...");
    hw_elevator.door_light(true);
    elevator.behaviour = ElevatorBehaviour::DoorOpen;

    thread::sleep(Duration::from_secs(3));

    println!("Closing door...");
    hw_elevator.door_light(false);
    elevator.behaviour = ElevatorBehaviour::Idle;
}



pub fn fsm_init(
    elevator: &mut Elevator,
    hw_elevator: &e::Elevator,
    floor_sensor_rx: &Receiver<u8>,
) {
    // BUG: should check that we are between floors. may also change the name of the function to signelize it may move the elevator to initialize.
    hw_elevator.motor_direction(e::DIRN_DOWN);

    println!("Waiting for floor sensor...");

    loop {
        match floor_sensor_rx.recv() {
            Ok(floor) => {
                println!("Received floor signal: {}", floor);
                elevator.floor = floor as i32;
                hw_elevator.motor_direction(e::DIRN_STOP);
                break;
            }
            Err(_) => {
                println!("No floor signal received yet...");
                thread::sleep(Duration::from_millis(1000));
            }
        }
    }

    println!("Elevator initialized at floor {}", elevator.floor);

    if elevator.floor != 0 {
        println!("Moving to floor 0...");
        elevator.dirn = Direction::Down;
        hw_elevator.motor_direction(e::DIRN_DOWN);

        loop {
            match floor_sensor_rx.recv() {
                Ok(floor) => {
                    if floor == 0 {
                        hw_elevator.motor_direction(e::DIRN_STOP);
                        println!("Arrived at floor 0");
                        break;
                    }
                }
                Err(_) => {
                    println!("No floor signal received while moving...");
                    thread::sleep(Duration::from_millis(100));
                }
            }
        }
    }

    elevator.dirn = Direction::Stop;
    elevator.behaviour = ElevatorBehaviour::Idle;

    println!("Elevator is ready for operation.");
}


pub fn fsm_init_lights(hw_elevator: &e::Elevator) {
    for i in 0..N_FLOORS {
        hw_elevator.call_button_light(i as u8, ButtonType::HallUp.to_u8(), false);
        hw_elevator.call_button_light(i as u8, ButtonType::HallDown.to_u8(), false);
        hw_elevator.call_button_light(i as u8, ButtonType::Cab.to_u8(), false);
    }
}
