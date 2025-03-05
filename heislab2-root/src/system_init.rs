use std::thread::*;
use std::time::*;

use crossbeam_channel as cbc;

use driver_rust::elevio;
use driver_rust::elevio::elev as e;

pub struct ElevatorController { 
    elevator: e::Elevator,
    call_button_rx: cbc::Receiver<elevio::poll::CallButton>,
    floor_sensor_rx: cbc::Receiver<u8>,
    stop_button_rx: cbc::Receiver<bool>,
    obstruction_rx: cbc::Receiver<bool>,
}

impl ElevatorController {
    pub fn new(port: &str, num_floors: u8) -> std::io::Result<Self> {
        let elevator = e::Elevator::init(port, num_floors)?;
        println!("Elevator started in constructor on port {}:\n{:#?}", port, elevator);

        let poll_period = Duration::from_millis(25);

        let (call_button_tx, call_button_rx) = cbc::unbounded::<elevio::poll::CallButton>();
        {
            let elevator = elevator.clone();
            spawn(move || elevio::poll::call_buttons(elevator, call_button_tx, poll_period));
        }

        let (floor_sensor_tx, floor_sensor_rx) = cbc::unbounded::<u8>();
        {
            let elevator = elevator.clone();
            spawn(move || elevio::poll::floor_sensor(elevator, floor_sensor_tx, poll_period));
        }

        let (stop_button_tx, stop_button_rx) = cbc::unbounded::<bool>();
        {
            let elevator = elevator.clone();
            spawn(move || elevio::poll::stop_button(elevator, stop_button_tx, poll_period));
        }

        let (obstruction_tx, obstruction_rx) = cbc::unbounded::<bool>();
        {
            let elevator = elevator.clone();
            spawn(move || elevio::poll::obstruction(elevator, obstruction_tx, poll_period));
        }

        Ok(ElevatorController {
            elevator,
            call_button_rx,
            floor_sensor_rx,
            stop_button_rx,
            obstruction_rx,
        })
    }
}