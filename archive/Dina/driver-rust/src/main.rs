use std::thread::*;
use std::time::*;
use crossbeam_channel as cbc;

use driver_rust::modules::elevator_object::*;
use alias_lib::{HALL_DOWN, HALL_UP, CAB, DIRN_DOWN, DIRN_UP, DIRN_STOP};
use elevator_init::Elevator;
use elevator_status_functions::Status;
use driver_rust::modules::order_object::order_init::Order;


// THIS IS SUPPOSED TO BE A SINGLE ELEVATOR MAIN THAT CAN RUN IN ONE THREAD


fn main() -> std::io::Result<()> {
    let elev_num_floors = 4;
    let mut elevator = Elevator::init("localhost:15657", elev_num_floors)?;
    println!("Elevator started:\n{:#?}", elevator);

    let poll_period = Duration::from_millis(25);

    let (call_button_tx, call_button_rx) = cbc::unbounded::<poll::CallButton>();
    {
        let elevator = elevator.clone();
        spawn(move || poll::call_buttons(elevator, call_button_tx, poll_period));
    }

    let (floor_sensor_tx, floor_sensor_rx) = cbc::unbounded::<u8>();
    {
        let elevator = elevator.clone();
        spawn(move || poll::floor_sensor(elevator, floor_sensor_tx, poll_period));
    }

    let (stop_button_tx, stop_button_rx) = cbc::unbounded::<bool>();
    {
        let elevator = elevator.clone();
        spawn(move || poll::stop_button(elevator, stop_button_tx, poll_period));
    }

    let (obstruction_tx, obstruction_rx) = cbc::unbounded::<bool>();
    {
        let elevator = elevator.clone();
        spawn(move || poll::obstruction(elevator, obstruction_tx, poll_period));
    }

    let mut dirn = DIRN_DOWN;


    if elevator.floor_sensor().is_none() {
        &elevator.motor_direction(dirn);
    }

    loop {
        cbc::select! {
            //tror denne kan bli
            recv(call_button_rx) -> a => {
                let call_button = a.unwrap();
                println!("{:#?}", call_button);
                elevator.call_button_light(call_button.floor, call_button.call, true);
                let new_order = Order::init(call_button.floor,call_button.call);
                elevator.add_to_queue(new_order);
                if &elevator.status == &(Status::Idle){
                    elevator.go_next_floor();
                }

                
            },

            recv(floor_sensor_rx) -> a => {
                let floor = a.unwrap();
                elevator.current_floor = floor;
                println!("Floor: {:#?}", floor);
                elevator.go_next_floor();
                
            },

            /*Burde nok modifiseres*/
            recv(stop_button_rx) -> a => {
                let stop = a.unwrap();
                println!("Stop button: {:#?}", stop);
                elevator.set_status(Status::Error);
                for f in 0..elev_num_floors {
                    for c in 0..3 {
                        elevator.call_button_light(f, c, false);
                    }
                }
                

            },
            recv(obstruction_rx) -> a => {
                let obstr = a.unwrap();
                println!("Obstruction: {:#?}", obstr);
                elevator.motor_direction(if obstr { DIRN_STOP } else { dirn });
                
            },
        }
    }
}
