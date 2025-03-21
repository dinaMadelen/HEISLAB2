use std::thread::*;
use std::time::*;
use crossbeam_channel as cbc;
use std::sync::{Arc, Mutex};
use std::net::{SocketAddr, IpAddr, Ipv4Addr};

use heislab2_root::modules::elevator_object::*;
use alias_lib::{DIRN_DOWN, DIRN_STOP};
use elevator_init::Elevator;
use heislab2_root::modules::system_status::SystemState;

use heislab2_root::modules::cab::*;
use cab::Cab;
use elevator_status_functions::Status;
use heislab2_root::modules::order_object::order_init::Order;


use heislab2_root::modules::master::master;
use heislab2_root::modules::slave::slave;

use heislab2_root::modules::udp::udp::{MessageType, UdpHeader, UdpMsg};

fn main() -> std::io::Result<()> {
    //--------------INIT ELEVATOR------------
    let elev_num_floors = 4;
    let mut elevator = Elevator::init("localhost:15657", elev_num_floors)?;

    println!("Elevator started:\n{:#?}", elevator);
    //--------------INIT ELEVATOR FINISH------------

    // --------------INIT CAB---------------
    let mut system_state = SystemState {
        me_id: 1,  // Example ID for this elevator
        master_id: 0, // Example master ID
        last_lifesign: Arc::new(Mutex::new(Instant::now())), // Set the initial timestamp
        active_elevators: Arc::new(Mutex::new(Vec::new())), // Empty list of active elevators
        failed_orders: Arc::new(Mutex::new(Vec::new())), // Empty list of failed orders
        sent_messages: Arc::new(Mutex::new(Vec::new())), // Empty list of sent messages
    };

    let inn_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 3500);
    let out_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 3600);
    let num_floors = 10;
    let set_id = 1; // Assign ID matching state.me_id for local IP assignment

    let mut cab = Cab::init(&inn_addr, &out_addr, num_floors, set_id, &mut system_state)?;

    println!("Cab initialized:\n{:#?}", elevator);

    // --------------INIT CAB FINISH---------------
    //---------------------------------------
    //Create Mutex for elevators
    //let elevators = Arc::new(Mutex::new(Vec::<Elevator>::new()));
    //---------------------------------------

    // --------------INIT CHANNELS---------------
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

    let (door_tx, door_rx) = cbc::unbounded::<bool>();

    // --------------INIT CHANNELS FINISHED---------------

    // --------------INIT RECIEVER THREAD------------------
    // -------------INIT RECIEVER FINISHED-----------------
    

    let dirn = DIRN_DOWN;

    if elevator.floor_sensor().is_none() {
        elevator.motor_direction(dirn);
    }

    

    loop {
        cbc::select! {
            recv(door_rx) -> a => {
                let door_signal = a.unwrap();
                if door_signal {
                    cab.set_status(Status::DoorOpen,elevator.clone());
                    cab.go_next_floor(door_tx.clone(),obstruction_rx.clone(),elevator.clone());
                    elevator.door_light(false);
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
                //make_udp_msg(cab.id, MessageType::NewOrder, );
                cab.add_to_queue(new_order);
                cab.turn_on_queue_lights(elevator.clone());

                //Safety if elevator is idle to double check if its going to correct floor
                if cab.status == Status::Idle{
                    cab.go_next_floor(door_tx.clone(),obstruction_rx.clone(),elevator.clone());
                }  
            },

            recv(floor_sensor_rx) -> a => {
                let floor = a.unwrap();
                println!("Floor: {:#?}", floor);

                //update current floor status
                cab.current_floor = floor;
                /*
                make_Udp_msg(elevator,message_type::Wordview) //guess this is the ping form
                */
                cab.go_next_floor(door_tx.clone(),obstruction_rx.clone(),elevator.clone());
                
            },

            /*Burde nok modifiseres*/
            recv(stop_button_rx) -> a => {
                let stop = a.unwrap();
                println!("Stop button: {:#?}", stop);
                
                cab.set_status(Status::Stop, elevator.clone());
                cab.turn_off_lights(elevator.clone());
                //broadcast current floor, stop and current queue - this might be redistributed
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
