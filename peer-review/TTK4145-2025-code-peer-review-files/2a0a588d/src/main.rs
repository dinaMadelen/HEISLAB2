use std::thread::*;
use std::time::*;

use crossbeam_channel as cbc;

use driver_rust::elevio;
use driver_rust::elevio::elev as e;
use driver_rust::elevio::poll::CallButton;

use driver_rust::UDP;
use driver_rust::FSM;
use driver_rust::state_manager;
use driver_rust::state_utils::fsm_state;


fn main() -> std::io::Result<()> {
    let elev_num_floors = 4;
    let elevator = e::Elevator::init("localhost:12345", elev_num_floors)?;
    println!("Elevator started:\n{:#?}", elevator);

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

    let (order_fulfilled_tx, order_fulfilled_rx) = cbc::unbounded::<CallButton>();
    let (from_fsm_tx, from_fsm_rx) = cbc::unbounded::<fsm_state>();
    let (from_state_manager_tx, from_state_manager_rx) = cbc::unbounded::<fsm_state>();
    let (uplink_tx, uplink_rx) = cbc::unbounded::<UDP::AllEncompassingDataType>();
    let (downlink_tx, downlink_rx) = cbc::unbounded::<UDP::AllEncompassingDataType>();

    spawn(move || UDP::udp_main(downlink_tx, uplink_rx));
    
    spawn(move || state_manager::state_manager_main(call_button_rx,
                                                    order_fulfilled_rx,
                                                    from_fsm_rx,
                                                    downlink_rx,
                                                    uplink_tx,
                                                    from_state_manager_tx));

    spawn(move || FSM::fsm_main(elevator.clone(),
                                from_state_manager_rx,
                                from_fsm_tx,
                                order_fulfilled_tx,
                                floor_sensor_rx,
                                obstruction_rx,
                                stop_button_rx));

    loop {
        sleep(Duration::from_secs(1));
    }
}

