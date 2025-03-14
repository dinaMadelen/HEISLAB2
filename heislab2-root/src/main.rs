use std::thread::*;
use std::time::*;
use crossbeam_channel as cbc;

use heislab2_root::modules::elevator_object::*;
use alias_lib::{HALL_DOWN, HALL_UP, CAB, DIRN_DOWN, DIRN_UP, DIRN_STOP};
use elevator_init::Elevator;
use elevator_status_functions::Status;
use heislab2_root::modules::order_object::order_init::Order;


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

                //Light turned on for correct lamp
                elevator.call_button_light(call_button.floor, call_button.call, true);

                //Make new order and add that order to elevators queue
                let new_order = Order::init(call_button.floor,call_button.call);

                //if the order is a cab call
                //Add to own queue
                //broadcast addition, but since order is in own cab the others taking over will not help  
                elevator.add_to_queue(new_order);

                //if order is a hall call
                //broadcast order 

                //Safety if elevator is idle to double check if its going to correct floor
                if &elevator.status == &(Status::Idle){
                    elevator.go_next_floor();
                }

                
            },

            recv(floor_sensor_rx) -> a => {
                let floor = a.unwrap();
                println!("Floor: {:#?}", floor);

                //update current floor status
                elevator.current_floor = floor;
                
                //broadcast current floor -this should also function as a ping

                //keep following current route
                elevator.go_next_floor();
                
            },

            /*Burde nok modifiseres*/
            recv(stop_button_rx) -> a => {
                let stop = a.unwrap();
                println!("Stop button: {:#?}", stop);
                
                elevator.set_status(Status::Stop);

                //broadcast current floor, stop and current queue - this might be redistributed

                //turn of lights
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

                //broadcast obstruction
                
            },

            //recv UDP message

            //check message type 
            //if messafe is from master
            

            //if order is yours
            //add to own queue

            //if order is someone elses
            //add to full queue

            //if message is from slave 
            //if order, add to own full queue and world view
            //if message is regarding dead elevator update elevators alive
            //if message is 
        }
    }
}
