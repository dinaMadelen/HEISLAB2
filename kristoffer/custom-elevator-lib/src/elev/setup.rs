#![allow(dead_code)]

use std::thread::*;
use std::time::*;

use crossbeam_channel as cbc;

use driver_rust::elevio;
use driver_rust::elevio::elev as e;

pub struct ElevatorSystem {
    pub elevator: e::Elevator,
    pub call_button_rx: cbc::Receiver<elevio::poll::CallButton>,
    pub floor_sensor_rx: cbc::Receiver<u8>,
    pub stop_button_rx: cbc::Receiver<bool>,
    pub obstruction_rx: cbc::Receiver<bool>,
}

pub fn create_elevator(tcp_port_num: &str, num_floors: u8) -> std::io::Result<ElevatorSystem> {
    let address = format!("localhost:{}", tcp_port_num);
    let elevator = e::Elevator::init(&address, num_floors)?;
    println!("Elevator started:\n{:#?}", elevator);

    let poll_period = Duration::from_millis(25);

    let (call_button_tx, call_button_rx) = cbc::unbounded::<elevio::poll::CallButton>();
    {
        let elev = elevator.clone();
        spawn(move || elevio::poll::call_buttons(elev, call_button_tx, poll_period));
    }

    let (floor_sensor_tx, floor_sensor_rx) = cbc::unbounded::<u8>();
    {
        let elev = elevator.clone();
        spawn(move || elevio::poll::floor_sensor(elev, floor_sensor_tx, poll_period));
    }

    let (stop_button_tx, stop_button_rx) = cbc::unbounded::<bool>();
    {
        let elev = elevator.clone();
        spawn(move || elevio::poll::stop_button(elev, stop_button_tx, poll_period));
    }

    let (obstruction_tx, obstruction_rx) = cbc::unbounded::<bool>();
    {
        let elev = elevator.clone();
        spawn(move || elevio::poll::obstruction(elev, obstruction_tx, poll_period));
    }

    let mut dirn = e::DIRN_DOWN;
    if elevator.floor_sensor().is_none() {
        elevator.motor_direction(dirn);
    }

    Ok(ElevatorSystem {
        elevator,
        call_button_rx,
        floor_sensor_rx,
        stop_button_rx,
        obstruction_rx,
    })
}

// pub fn listen_for_orders(elevator: e::Elevator) {
//     //------------------------
//     // Start button listening
//     //------------------------
//     loop {
//         cbc::select! {
//             recv(call_button_rx) -> a => {
//                 let call_button = a.unwrap();
//                 println!("{:#?}", call_button);
//                 elevator.call_button_light(call_button.floor, call_button.call, true);
//             },
//             recv(floor_sensor_rx) -> a => {
//                 let floor = a.unwrap();
//                 println!("Floor: {:#?}", floor);
//                 dirn =
//                     if floor == 0 {
//                         e::DIRN_UP
//                     } else if floor == elev_num_floors-1 {
//                         e::DIRN_DOWN
//                     } else {
//                         dirn
//                     };
//                 elevator.motor_direction(dirn);
//             },
//             recv(stop_button_rx) -> a => {
//                 let stop = a.unwrap();
//                 println!("Stop button: {:#?}", stop);
//                 for f in 0..elev_num_floors {
//                     for c in 0..3 {
//                         elevator.call_button_light(f, c, false);
//                     }
//                 }
//             },
//             recv(obstruction_rx) -> a => {
//                 let obstr = a.unwrap();
//                 println!("Obstruction: {:#?}", obstr);
//                 elevator.motor_direction(if obstr { e::DIRN_STOP } else { dirn });
//             },
//         }
//     }

// }

