use std::thread::*;
use std::time::*;
use crossbeam_channel as cbc;
use std::sync::{Arc, Mutex};
use std::net::{SocketAddr, IpAddr, Ipv4Addr};

use heislab2_root::modules::elevator_object::*;
use alias_lib::{DIRN_DOWN, DIRN_STOP};
use elevator_init::Elevator;
use heislab2_root::modules::*;
use system_status::SystemState;

use cab_object::*;
use cab::Cab;
use cab::Role;
use elevator_status_functions::Status;
use order_object::order_init::Order;


use master_functions::master::*;
use slave_functions::slave::*;
use system_init::*;


use heislab2_root::modules::udp_functions::udp::*;
use udp_functions::udp::UdpData;

fn main() -> std::io::Result<()> {
    //--------------INIT ELEVATOR------------

// Check boot function in system Init

    let elev_num_floors = 4;
    let mut elevator = Elevator::init("localhost:15657", elev_num_floors)?;

    //Dummy message to have an empty message in current worldview 
    let boot_worldview =  UdpMsg {
        header: UdpHeader {
            sender_id: 0,
            message_type: MessageType::Worldview,
            checksum: vec![0],
        },
        data: UdpData::None,
    };

    println!("Elevator started:\n{:#?}", elevator);
    //--------------INIT ELEVATOR FINISH------------

    // --------------INIT CAB---------------
    let mut system_state = Arc::new(boot());

    //OBS!!! This is localhost, aka only localy on the computer, cant send between computers on tha same net, check Cab.rs
    //let new_cab = Cab::init(&inn_addr, &out_addr, 4, 2, &mut state)?;

    let inn_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 3500);
    let out_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 3600);
    let set_id = system_state.me_id; // Assign ID matching state.me_id for local IP assignment

    let mut cab = Cab::init(&inn_addr, &out_addr, elev_num_floors, set_id, &system_state)?;

    println!("Cab initialized:\n{:#?}", elevator);

    // --------------INIT CAB FINISH---------------
    //---------------INIT UDP HANDLER-------------------
    let mut udphandler = init_udp_handler(cab.clone());

    //-------------INIT UDP HANDLER FINISH-----------------

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
    let (master_update_tx, master_update_rx) = cbc::unbounded::<Vec<Cab>>();
    let (order_update_tx, order_update_rx) = cbc::unbounded::<Vec<Order>>();

    // --------------INIT CHANNELS FINISHED---------------

    // --------------INIT RECIEVER THREAD------------------
    let mut system_state_clone = Arc::clone(&system_state);

    spawn(move||
        loop{
            udphandler.receive(5, &system_state_clone);
        }
    );
    // -------------INIT RECIEVER FINISHED-----------------


    let dirn = DIRN_DOWN;

    if elevator.floor_sensor().is_none() {
        elevator.motor_direction(dirn);
    }

    let msg = make_udp_msg(cab.id, MessageType::NewOnline, UdpData::Cab(cab.clone()));
    udp_broadcast(&msg);
    

    // ------------------ MAIN LOOP ---------------------
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
                //broadcast addition, but since order is in own cab the others taking over will not help
                let new_order = Order::init(call_button.floor, call_button.call);
                {   
                    //Broadcast new request
                    /*let msg = make_udp_msg(cab.id, MessageType::NewRequest, UdpData::Order(new_order.clone()));*/
                    udp_broadcast(&msg);

                    // IF MASTER SORT ELEVATORS AND GIVE ORDER
                    if new_order.order_type == CAB{
                        cab.add_to_queue(new_order);

                    } else if cab.role==Role::Master{
                        let system_state_active_elevators = system_state.active_elevators.lock().unwrap();
                        let best_elevator_vec = best_to_worst_elevator(&new_order,&*system_state_active_elevators);
                        drop(system_state_active_elevators);
                        
                        //Give order and broadcast new worldview
                        if let Some(best_elevator) = best_elevator_vec.first() { 
                            give_order(*best_elevator,vec![&new_order], &system_state, &udphandler);
                            master_worldview(&system_state);
                        }
                    }

                }

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
