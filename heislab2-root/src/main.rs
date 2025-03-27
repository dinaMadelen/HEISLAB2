use std::thread::*;
use std::time::*;
use crossbeam_channel as cbc;
use heislab2_root::modules::cab_object::cab_wrapper::add_cab_to_sys_state;
use heislab2_root::modules::master_functions::master_wrapper;
use heislab2_root::modules::udp_functions::udp_wrapper::spawn_udp_reciever_thread;
//use heislab2_root::modules::io::io_init;
//use heislab2_root::modules::master_functions::master::handle_slave_failure;
use std::sync::Arc;
use std::net::{SocketAddr, IpAddr, Ipv4Addr};

use heislab2_root::modules::elevator_object::*;
use alias_lib::{DIRN_DOWN, DIRN_STOP};
use elevator_init::Elevator;
use heislab2_root::modules::*;


use cab_object::*;
use cab::Cab;
use elevator_status_functions::Status;
use order_object::order_init::Order;
use slave_functions::slave::*;
use master_functions::master::*;
use system_init::*;


use heislab2_root::modules::udp_functions::udp::*;
use udp_functions::udp::UdpData;

use heislab2_root::modules::io::io_init::*;

use heislab2_root::modules::udp_functions::udp_wrapper::*;
use heislab2_root::modules::elevator_object::elevator_wrapper::*;
use heislab2_root::modules::cab_object::cab_wrapper::*;


fn main() -> std::io::Result<()> {
    //----------------
    // Initialization
    //----------------
    // initialize elevator
    let elev_num_floors = 4;
    let elevator = Elevator::init("localhost:15657", elev_num_floors)?;
    println!("Elevator started:\n{:#?}", elevator);

    // create dummy empty worldview message 
    let boot_worldview = udp_wrapper::create_empty_worldview_msg();

    // initialize system state
    let system_state = initialize_system_state();

    // create socket addresses
    let inn_addr = udp_wrapper::create_socket_address(3700, system_state.me_id);
    let out_addr = udp_wrapper::create_socket_address(3800, system_state.me_id);

    // initialize cab
    let mut cab = cab_wrapper::initialize_cab(elev_num_floors, &system_state, elevator.clone(), 3700, 3800)?;

    // initialize udp handler
    let udphandler = initialize_udp_handler(cab.clone());

    // add cab to system state 
    add_cab_to_sys_state(sysstem_state.clone(), cab)?;

    // initialize io channels
    let io_channels = IoChannels::new(&elevator);

    // clone system state
    let system_state_clone = system_state.clone();
    
    // set master id
    master_wrapper::set_master_id(system_state.clone())?;

    // spawn udp reciever
    udp_wrapper::spawn_udp_reciever_thread(udphandler.clone(), system_state.clone(), io_channels.clone());

    // go down until the elevator finds a floor
    elevator_wrapper::go_down_until_floor_found(&mut elevator, DIRN_DOWN);
    
    // spawn thread that moniors for dead elevator and broadcasts worldview
    elevator_wrapper::spawn_elevator_monitor_thread(system_state.clone(), udp_handler.clone());

    // print master ID
    master_wrapper::print_master_id(system_state.clone());

    // broadcast i am alive message
    udp_wrapper::broadcast_alive_msg(udphandler.clone(), system_state.clone());

    // spawn thread that monitors for master failure
    master_wrapper::spawn_master_failure_check_thread(system_state.clone(), udp_handler.clone());
    
    // spawn a loop that makes sure that the elevators alway finish their queues 
    elevator_wrapper::spawn_queue_finish_thread(
        system_state.clone(),
        udphandler.clone(),
        elevator.clone(),
        io_channels.clone()
    );

    // ------------------ MAIN LOOP ---------------------
    loop {
        cbc::select! {
            
            recv(io_channels.light_update_rx) -> a => {
                //Turn onn all lights in own queue
                let mut known_elevators_locked = system_state.known_elevators.lock().unwrap();
                known_elevators_locked.get_mut(0).unwrap().turn_on_just_lights_in_queue(elevator.clone() );
                drop(known_elevators_locked);
            },

            recv(io_channels.order_update_rx) -> a => {
                //ASSUME THE ORDER ALREADY IS ADDED TO QUEUE
                //Mulig denne er for tidlig
                let mut known_elevators_locked = system_state.known_elevators.lock().unwrap();
                if known_elevators_locked.is_empty(){

                }else{
                    println!("current queue: {:?}",known_elevators_locked.get_mut(0).unwrap().queue);
                    let cab_clone = known_elevators_locked.get(0).unwrap().clone();
                    if known_elevators_locked.get_mut(0).unwrap().status == Status::Idle {
                        let imalive = make_udp_msg(system_state.me_id, MessageType::ImAlive, UdpData::Cab(cab_clone));
                        for elevator in known_elevators_locked.iter(){
                            udphandler.send(&elevator.inn_address, &imalive);
                        }
                        //udp_broadcast(&imalive);
                    }
                    known_elevators_locked.get_mut(0).unwrap().go_next_floor(io_channels.door_tx.clone(),io_channels.obstruction_rx.clone(),elevator.clone());
                    drop(known_elevators_locked);
                }
            },
            
            recv(io_channels.door_rx) -> a => {
                let door_signal = a.unwrap();
                if door_signal {
                        elevator.door_light(false);
                        let mut known_elevators_locked = system_state.known_elevators.lock().unwrap();
                        known_elevators_locked.get_mut(0).unwrap().set_status(Status::Idle, elevator.clone());
                        let cab_clone = known_elevators_locked.get(0).unwrap().clone();
                        if !cab_clone.queue.is_empty(){
                            if cab_clone.current_floor == (cab_clone.queue.get(0)).unwrap().floor{
                                known_elevators_locked.get_mut(0).unwrap().queue.remove(0);
                            }
                        }
                        let ordercomplete = make_udp_msg(system_state.me_id, MessageType::OrderComplete, UdpData::Cab(cab_clone.clone()));
                        if cab_clone.queue.is_empty(){
                        println!("No orders in this elevators queue");
                    }else {
                        
                        
                    }
                    drop(known_elevators_locked);
                    

                    let msg = make_udp_msg(system_state.me_id, MessageType::ImAlive, UdpData::Cab(cab_clone));
                    let mut known_elevators_locked = system_state.known_elevators.lock().unwrap();
                        for elevator in known_elevators_locked.iter(){
                            udphandler.send(&elevator.inn_address, &ordercomplete);
                            udphandler.send(&elevator.inn_address, &msg);
                        }
                    known_elevators_locked.get_mut(0).unwrap().go_next_floor(io_channels.door_tx.clone(),io_channels.obstruction_rx.clone(),elevator.clone());
                    drop(known_elevators_locked);
                                  
                    
                }
            },

            recv(io_channels.call_rx) -> a => {
                let call_button = a.unwrap();
                println!("{:#?}", call_button);
                //Make new order and add that order to elevators queue
                let new_order = Order::init(call_button.floor, call_button.call);
                {   
                    //Broadcast new request
                    let msg = make_udp_msg(system_state.me_id, MessageType::NewRequest, UdpData::Order(new_order.clone()));
                    let known_elevators_locked = system_state.known_elevators.lock().unwrap();
                        for elevator in known_elevators_locked.iter(){
                            udphandler.send(&elevator.inn_address, &msg);
                        }
                    drop(known_elevators_locked);
                   

                }

                //cab.turn_on_queue_lights(elevator.clone());
                let mut known_elevators_locked = system_state.known_elevators.lock().unwrap();

                //Safety if elevator is idle to double check if its going to correct floor
                if known_elevators_locked.is_empty(){
                    println!("No active elevators, not even this one ID:{}",system_state.me_id);

                }else if known_elevators_locked.get_mut(0).unwrap().status == Status::Idle{
                    known_elevators_locked.get_mut(0).unwrap().go_next_floor(io_channels.door_tx.clone(),io_channels.obstruction_rx.clone(),elevator.clone());
                    if known_elevators_locked.get_mut(0).unwrap().status == Status::Moving{
                        let msg = make_udp_msg(system_state.me_id, MessageType::ImAlive, UdpData::Cab(known_elevators_locked.get(0).unwrap().clone()));
                        for elevator in known_elevators_locked.iter(){
                            udphandler.send(&elevator.inn_address, &msg);
                        }
                    }
                } 
                drop(known_elevators_locked);
            },

            recv(io_channels.floor_rx) -> a => {
                let floor = a.unwrap();
                println!("Floor: {:#?}", floor);
                //update current floor status

                let mut known_elevators_locked = system_state.known_elevators.lock().unwrap();
                known_elevators_locked.get_mut(0).unwrap().current_floor = floor;
                drop(known_elevators_locked);

                //Do stuff
                let mut known_elevators_locked = system_state.known_elevators.lock().unwrap();
                known_elevators_locked.get_mut(0).unwrap().go_next_floor(io_channels.door_tx.clone(),io_channels.obstruction_rx.clone(),elevator.clone());
                known_elevators_locked.get_mut(0).unwrap().turn_on_just_lights_in_queue(elevator.clone());
                drop(known_elevators_locked);


                //Broadcast new state
                let  known_elevators_locked = system_state.known_elevators.lock().unwrap();
                let cab_clone = known_elevators_locked.get(0).unwrap().clone();
                let msg = make_udp_msg(system_state.me_id, MessageType::ImAlive, UdpData::Cab(cab_clone));
                    for elevator in known_elevators_locked.iter(){
                        udphandler.send(&elevator.inn_address, &msg);
                       }
                drop(known_elevators_locked);
                
            },

            /*Burde nok modifiseres*/
            recv(io_channels.stop_rx) -> a => {
                let stop = a.unwrap();
                println!("Stop button: {:#?}", stop);
                let mut known_elevators_locked = system_state.known_elevators.lock().unwrap();
                if known_elevators_locked.is_empty(){
                    println!("There are no elevators in the system")
                }else {
                    if known_elevators_locked.get(0).unwrap().status == Status::Stop{
                        known_elevators_locked.get_mut(0).unwrap().alive=true;
                        known_elevators_locked.get_mut(0).unwrap().set_status(Status::Idle, elevator.clone());
                        drop(known_elevators_locked);
                        let mut system_state_clone = Arc::clone(&system_state);
                        send_new_online(&system_state_clone);

                    }else{
                        known_elevators_locked.get_mut(0).unwrap().set_status(Status::Stop, elevator.clone());
                        
                        //WHO CONTROLS THE LIGHTS
                        let mut known_elevators_locked = system_state.known_elevators.lock().unwrap();
                        known_elevators_locked.get_mut(0).unwrap().turn_off_lights(elevator.clone());
                        drop(known_elevators_locked);
                        let mut system_state_clone = Arc::clone(&system_state);
                        send_error_offline(&system_state_clone);
                    }
                    



                }
                
            },

            recv(io_channels.obstruction_rx) -> a => {
                let obstr = a.unwrap();
                println!("Obstruction: {:#?}", obstr);
                //elevator.motor_direction(if obstr { DIRN_STOP } else { dirn });
                let mut known_elevators_locked = system_state.known_elevators.lock().unwrap();
                if known_elevators_locked.is_empty(){

                }else {
                    //Should add cab to systemstatevec and then broadcast new state of stopped
                    if obstr{
                        known_elevators_locked.get_mut(0).unwrap().set_status(Status::Obstruction,elevator.clone());
                    }else{
                        known_elevators_locked.get_mut(0).unwrap().set_status(Status::Idle,elevator.clone());
                        known_elevators_locked.get_mut(0).unwrap().go_next_floor(io_channels.door_tx.clone(),io_channels.obstruction_rx.clone(),elevator.clone());
                        known_elevators_locked.get_mut(0).unwrap().turn_on_just_lights_in_queue(elevator.clone());
                    }
                    drop(known_elevators_locked);
                }
            },
        }
    }
}
