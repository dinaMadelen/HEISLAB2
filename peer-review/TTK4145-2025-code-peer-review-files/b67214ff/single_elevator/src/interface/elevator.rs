use std::thread;

use crossbeam_channel as cbc;


use super::{ElevatorArgument, HallRequestMatrix};
use crate::elevio::poll::CallButton;
use crate::execution::{driver_communcation::elevator_driver_polling, elevator, fsm::FSM};

pub fn run_elevator(
    elevator_number:u8, 
    hall_request_driver_tx: cbc::Sender<(CallButton, bool)>, 
    elevator_argument_tx: cbc::Sender<(u8,ElevatorArgument)>, 
    hall_request_rx: cbc::Receiver<(u8, HallRequestMatrix)>,
    obstruction_switch_tx: cbc::Sender<bool>,
    elevator_port: String
){
    
    println!("Started elevator");

    //input device
    let (motor_direction_tx, motor_direction_rx) = cbc::unbounded();
    let (call_button_light_tx, call_button_light_rx) = cbc::unbounded();
    let (floor_indicator_tx, floor_indicator_rx) = cbc::unbounded();
    let (door_light_tx, door_light_rx) = cbc::unbounded();  

    //output device
    let (call_button_tx, call_button_rx) = cbc::unbounded();
    let (floor_sensor_tx, floor_sensor_rx) = cbc::unbounded();
    let (obstruction_tx, obstruction_rx) = cbc::unbounded();


    thread::spawn(|| {
        elevator_driver_polling (
            motor_direction_rx,
            call_button_light_rx,
            floor_indicator_rx,
            door_light_rx,

            call_button_tx,
            floor_sensor_tx,
            obstruction_tx,
            elevator_port
        );
    });

    thread::spawn(move || {
        let mut fsm: FSM = 
            FSM ::init(
                elevator::Elevator::elevator_init(),
                motor_direction_tx, 
                call_button_light_tx, 
                floor_indicator_tx,
                door_light_tx,
                
                call_button_rx,
                floor_sensor_rx, 
                obstruction_rx,

                elevator_number, 
                hall_request_driver_tx, 
                elevator_argument_tx, 
                hall_request_rx,
                obstruction_switch_tx,
            );
        fsm.run_fsm();
    });
    loop {}
}
