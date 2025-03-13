use crossbeam_channel::{Receiver, Sender};


use std::{thread, time::Duration};

use crossbeam_channel as cbc;

use crate::{elevio::{elev, poll::{self, CallButton}}, execution::elevator::N_FLOORS};


pub fn elevator_driver_polling (
    motor_direction_rx: Receiver<u8>,
    call_button_light_rx: Receiver<(u8,u8,bool)>,
    floor_indicator_rx: Receiver<u8>,
    door_light_rx: Receiver<bool>,

    call_button_tx: Sender<CallButton>,
    floor_sensor_tx: Sender<u8>,
    obstruction_tx: Sender<bool>,
    elevator_port: String

) {

    let elevator_driver = elev::ElevatorDriver::init(&elevator_port, N_FLOORS as u8).unwrap();

    let poll_period = Duration::from_millis(25);

    let (call_button_poll_tx, call_button_poll_rx) = cbc::unbounded::<poll::CallButton>();
    {
        let elevator_driver = elevator_driver.clone();
        thread::spawn(move || poll::call_buttons(elevator_driver, call_button_poll_tx, poll_period));
    }

    let (floor_sensor_poll_tx, floor_sensor_poll_rx) = cbc::unbounded::<u8>();
    {
        let elevator_driver = elevator_driver.clone();
        thread::spawn(move || poll::floor_sensor(elevator_driver, floor_sensor_poll_tx, poll_period));
    }

    let (obstruction_poll_tx, obstruction_poll_rx) = cbc::unbounded::<bool>();
    {
        let elevator_driver = elevator_driver.clone();
        thread::spawn(move || poll::obstruction(elevator_driver, obstruction_poll_tx, poll_period));
    }
    
    println!("Interface initialized");
    loop {

        cbc::select! {
            recv(call_button_poll_rx) -> cb => {
                call_button_tx.send(cb.unwrap()).expect("call_button_tx");

            },

            recv(floor_sensor_poll_rx) -> floor => {
                floor_sensor_tx.send(floor.unwrap()).expect("floor_sensor_tx");
            },

            recv(obstruction_poll_rx) -> obstr => {
                obstruction_tx.send(obstr.unwrap()).expect("obstruction_tx");
            },
            default() => (),
        }



        cbc::select! {
            recv(motor_direction_rx) -> md => {
                elevator_driver.motor_direction(md.unwrap());
            },
            recv(call_button_light_rx) -> cbl => {
                let (floor, btn, on) = cbl.unwrap();
                elevator_driver.call_button_light(floor,btn, on);
            },
            recv(floor_indicator_rx) -> floor => {
                elevator_driver.floor_indicator(floor.unwrap());
            },
            recv(door_light_rx) -> on => {
                elevator_driver.door_light(on.unwrap());
            },
            default() => (),
        }
    }



}