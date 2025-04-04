use crossbeam_channel as cbc;
use driver_rust::config::config;
use driver_rust::distributor;
use driver_rust::elevator_controller;
use driver_rust::elevator_controller::lights;
use driver_rust::elevator_controller::orders::{AllOrders, Orders};
use driver_rust::cost_function::cost_function::{assign_orders};
use driver_rust::elevio;
use driver_rust::elevio::elev as e;
use driver_rust::network;
use driver_rust::network::udp;
use std::sync::Arc;
use std::thread::*;
use std::env;

fn main() -> std::io::Result<()> {
    
    let (port, elevator_id) = fetch_command_line_args();

    //println!("{}, {}", port, elevator_id);

    let addr = format!("localhost:{}", port);

    let elevator = e::Elevator::init(&addr, config::ELEV_NUM_FLOORS)?;
    //println!("Elevator started:\n{:#?}", elevator);

    let (new_order_tx, new_order_rx) = cbc::unbounded::<Orders>();
    let (emergency_reset_tx, emergency_reset_rx) = cbc::unbounded::<bool>();
    let (new_state_tx, new_state_rx) = cbc::unbounded::<elevator_controller::elevator_fsm::State>();
    let (order_completed_tx, order_completed_rx) = cbc::unbounded::<elevio::poll::CallButton>();

    {
        let elevator = elevator.clone();
        spawn(move || {
            elevator_controller::elevator_fsm::elevator_fsm(
                &elevator,
                new_order_rx,
                order_completed_tx,
                emergency_reset_tx,
                &new_state_tx,
            )
        });
    }

    {
        let elevator = elevator.clone();
        spawn(move || {
            distributor::distributor::distributor(
                &elevator,
                elevator_id,
                new_state_rx,
                order_completed_rx,
                new_order_tx,
            )
        });
    }

    // let mut all_orders = AllOrders::init();

    loop {
        // cbc::select! {
        // recv(call_button_rx) -> a => {
        //     let call_button = a.unwrap();
        //     all_orders.add_order(call_button, config::elevator_id as usize, &new_order_tx);
        // },
        // recv(order_completed_rx) -> a => {
        //     let call_button = a.unwrap();
        //     all_orders.remove_order(call_button, config::elevator_id as usize, &new_order_tx);
        // },
        //     recv(emergency_reset_rx) -> _ => {
        //         all_orders = AllOrders::init();
        //         new_order_tx.send(all_orders.orders).unwrap();
        //     }
        // }
    }
    Ok(())
}



pub fn fetch_command_line_args() -> (u16, u8) {

    let default_port = 15657;
    let default_elevator_id = 0;

    let command_line_args: Vec<String> = env::args().collect();

    let port = if command_line_args.len() > 1 {
        match command_line_args[1].parse::<u16>() {
            Ok(p) => p,
            Err(_) => {
                //println!("Warning: Invalid port provided. Using default: {}", default_port);
                default_port
            }
        }
    } else {
        default_port
    };

    let elevator_id = if command_line_args.len() > 2 {
        match command_line_args[2].parse::<u8>() {
            Ok(id) => id,
            Err(_) => {
                //println!("Warning: Invalid elevator ID provided. Using default: {}", default_elevator_id);
                default_elevator_id
            }
        }
    } else {
        default_elevator_id
    };

    (port, elevator_id)
}
