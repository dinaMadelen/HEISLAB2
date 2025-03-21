use std::thread::*;
use std::time::*;
use crossbeam_channel as cbc;
//use std::sync::{Arc, Mutex};

use heislab2_root::modules::elevator_object::*;
use alias_lib::{DIRN_DOWN, DIRN_STOP};
use elevator_init::Elevator;
use elevator_status_functions::Status;
use heislab2_root::modules::order_object::order_init::Order;
use heislab2_root::modules::master::master;
use heislab2_root::modules::slave::slave;
use heislab2_root::modules::udp::udp;


// THIS IS SUPPOSED TO BE A SINGLE ELEVATOR MAIN THAT CAN RUN IN ONE THREAD

fn main() -> std::io::Result<()> {
    //Dummy Variables
    let elev_num_floors = 4;


    let mut elevator = Elevator::init("localhost:15657", elev_num_floors)?;
    println!("Elevator started:\n{:#?}", elevator);
 
    //---------------------------------------
    //Create Mutex for elevators
    //let elevators = Arc::new(Mutex::new(Vec::<Elevator>::new()));
    //---------------------------------------

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

    let dirn = DIRN_DOWN;


    if elevator.floor_sensor().is_none() {
        elevator.motor_direction(dirn);
    }

    let (door_tx, door_rx) = cbc::unbounded::<bool>();

    let (status_tx, status_rx) = cbc::unbounded::<Status>();
    let (req_status_tx, req_status_rx) = cbc::unbounded::<bool>();

    let (queue_tx, queue_rx) = cbc::unbounded::<Vec<Order>>();
    let (req_queue_tx, req_queue_rx) = cbc::unbounded::<bool>();
    

    loop {
        cbc::select! {
            recv(door_rx) -> a => {
                let door_signal = a.unwrap();
                if door_signal {
                    elevator.go_next_floor(door_tx.clone(),obstruction_rx.clone());
                }
            },

            recv(call_button_rx) -> a => {
                let call_button = a.unwrap();
                println!("{:#?}", call_button);
                //Make new order and add that order to elevators queue
                let new_order = Order::init(call_button.floor,call_button.call);
                
                //broadcast addition, but since order is in own cab the others taking over will not help
                /*
                make_Udp_msg(elevator, message_type::New_Order); //Broadcasts the new order so others can update wv

                if call_button.call == CAB{
                    elevator.add_to_queue(new_order);
                }
                */
                elevator.add_to_queue(new_order);
                elevator.turn_on_queue_lights();

                //Safety if elevator is idle to double check if its going to correct floor
                let true_status= elevator.status.lock().unwrap();
                let clone_true_status = true_status.clone();
                drop(true_status);

                if clone_true_status == Status::Idle{
                    elevator.go_next_floor(door_tx.clone(),obstruction_rx.clone());
                }  
            },

            recv(floor_sensor_rx) -> a => {
                let floor = a.unwrap();
                println!("Floor: {:#?}", floor);

                //update current floor status
                elevator.current_floor = floor;
                /*
                make_Udp_msg(elevator,message_type::Wordview) //guess this is the ping form
                */
                
                //keep following current route
                elevator.go_next_floor(door_tx.clone(),obstruction_rx.clone());
                
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
            //if message is from master
            // MAKE HANDLE MASTER MESSAGE FUNCTION
            

            //if order is yours
            //add to own queue

            //if order is someone elses
            //add to full queue

            //if message is from slave 
            //if order, add to own full queue and world view
            //if message is an ack update elevators alive
            
        }
    }
}
