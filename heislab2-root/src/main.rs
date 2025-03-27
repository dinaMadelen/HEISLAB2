use std::thread::*;
use std::time::*;
use crossbeam_channel as cbc;
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



fn main() -> std::io::Result<()> {
    //--------------INIT ELEVATOR------------
    // Check boot function in system Init
    let elev_num_floors = 4;
    let elevator = Elevator::init("localhost:15000", elev_num_floors)?;

    //Dummy message to have an empty message in current worldview 
    let boot_worldview =  UdpMsg {
        header: UdpHeader {
            sender_id: 0,
            message_type: MessageType::Worldview,
            checksum: 0,
        },
        data: UdpData::Checksum(0),
    };

    println!("Elevator started:\n{:#?}", elevator);

    //--------------INIT ELEVATOR FINISH------------

    // --------------INIT CAB---------------
    let system_state = Arc::new(boot());

    //OBS!!! This is localhost, aka only localy on the computer, cant send between computers on tha same net, check Cab.rs
    //let new_cab = Cab::init(&inn_addr, &out_addr, 4, 2, &mut state)?;
    
    let inn_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 3700 + system_state.me_id as u16);
    let out_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 3800 + system_state.me_id as u16);
    
    let set_id = system_state.me_id; // Assign ID matching state.me_id for local IP assignment
    println!("me id is {}",system_state.me_id);
    //Make free cab
    let mut cab = Cab::init(&inn_addr, &out_addr, elev_num_floors, set_id, &system_state)?;
    cab.turn_off_lights(elevator.clone());

    //---------------INIT UDP HANDLER-------------------
    let udphandler = Arc::new(init_udp_handler(cab.clone()));
    //-------------INIT UDP HANDLER FINISH-----------------

    //Lock free cab into captivity :(
    let mut known_elevators_locked = system_state.known_elevators.lock().unwrap();
    known_elevators_locked.push(cab);
    drop(known_elevators_locked);

    println!("Cab initialized:\n{:#?}", elevator);

    // --------------INIT CAB FINISH---------------
    
    // --------------INIT CHANNELS---------------
    let io_channels = IoChannels::new(&elevator);
    // --------------INIT CHANNELS FINISHED---------------

    // --------------INIT RECIEVER THREAD------------------
    let system_state_clone = Arc::clone(&system_state);
    
    // -------------------SET MASTER ID------------------
    let mut known_elevators_locked = system_state.known_elevators.lock().unwrap();
    let mut cab_clone = known_elevators_locked.get_mut(0).unwrap().clone();
    drop(known_elevators_locked);

    set_new_master(&mut cab_clone, &system_state);
    
    // -------------------SET MASTER ID FINISHED------------------


    // -------------INIT RECIEVER-----------------
    let udphandler_clone = Arc::clone(&udphandler);
    let order_update_clone = io_channels.order_update_tx.clone();
    let light_update_clone = io_channels.light_update_tx.clone();
    spawn(move||{
        loop{
            let handler = Arc::clone(&udphandler_clone); 
            handler.receive(60000, &system_state_clone, order_update_clone.clone(), light_update_clone.clone());
        }
    });
    // -------------INIT RECIEVER FINISHED-----------------

    
    let dirn = DIRN_DOWN;
    if elevator.floor_sensor().is_none() {
        elevator.motor_direction(dirn);
    }
    let master_id_clone = system_state.master_id.lock().unwrap().clone();
    println!("The master is assigned as: {}",master_id_clone);

     //SEND MESSAGE TO EVERYONE THAT YOU ARE ALIVE
    let  known_elevators_locked = system_state.known_elevators.lock().unwrap();
    let cab_clone = known_elevators_locked.get(0).unwrap().clone();
    drop(known_elevators_locked);
   
    let msg = make_udp_msg(system_state.me_id, MessageType::NewOnline, UdpData::Cab(cab_clone));
    let known_elevators_locked = system_state.known_elevators.lock().unwrap();
    for port in 3701..3705{
        let inn_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),port as u16);
        udphandler.send(&inn_addr, &msg);
    }
    drop(known_elevators_locked);
    // END SEND I ALIVE

    //ELEVATORMONITOR!!!
    let system_state_clone = Arc::clone(&system_state);
    let udp_handler_clone = Arc::clone(&udphandler);
    spawn(move||{
            loop{
                fix_master_issues(&system_state_clone, &udp_handler_clone);

                sleep(Duration::from_secs(1));
                let now = SystemTime::now();
                
                // Iterate in reverse order so that removing elements doesn't affect things
                {
                    let  known_elevators_locked = system_state_clone.known_elevators.lock().unwrap();
                    for i in (0..known_elevators_locked.len()).rev() {
                        let elevator = &known_elevators_locked[i];
                        // Only check elevators that are Moving or DoorOpen.
                        if elevator.status == Status::Moving || elevator.status == Status::DoorOpen {
                            if let Ok(elapsed) = now.duration_since(elevator.last_lifesign) {
                                if elapsed >= Duration::from_secs(10) {
                                    let dead_elevator = known_elevators_locked.get(i).unwrap();
                                    println!("Elevator {} is dead (elapsed: {:?})", dead_elevator.id, elapsed);
                                    let msg = make_udp_msg(system_state_clone.me_id, MessageType::ErrorOffline, UdpData::Cab(dead_elevator.clone()));
                                    for elevator in known_elevators_locked.iter(){
                                        udp_handler_clone.send(&elevator.inn_address, &msg);
                                    }
                                }
                            }
                        }
                    }
                }
                

                {   
                    let known_elevators_locked = system_state_clone.known_elevators.lock().unwrap();
                    let locked_master_id = system_state_clone.master_id.lock().unwrap();
                    if system_state_clone.me_id == *locked_master_id{
                        print!("BROADCASTING WORLDVIEW _____________________");
                        //MASTER WORLDVIEW BROADCAST
                        let worldview = make_udp_msg(system_state_clone.me_id, MessageType::Worldview, UdpData::Cabs(known_elevators_locked.clone()));
                        for elevator in known_elevators_locked.iter(){
                            udp_handler_clone.send(&elevator.inn_address, &worldview);
                        }
                        //master_worldview(&system_state_clone);
                    }
                }
                sleep(Duration::from_secs(1));
                check_master_failure(&system_state_clone, &udp_handler_clone);
            }
    });


    //STARTING A LOOP TO MAKE SURE ALL ELEVATORS ALWAYS FINISH THEIR QUEUE
    let system_state_clone = Arc::clone(&system_state);
    let elevator_clone = elevator.clone();
    let door_tx_clone = io_channels.door_tx.clone();
    let obstruction_tx_clone = io_channels.obstruction_rx.clone();
    spawn(move|| {
        loop{
            sleep(Duration::from_millis(100));
            
            let mut known_elevators_locked = system_state_clone.known_elevators.lock().unwrap();
            if !known_elevators_locked.get_mut(0).unwrap().queue.is_empty(){
                known_elevators_locked.get_mut(0).unwrap().go_next_floor(door_tx_clone.clone(),obstruction_tx_clone.clone() ,elevator_clone.clone());
                let all_orders = system_state_clone.all_orders.lock().unwrap().clone();
                known_elevators_locked.get_mut(0).unwrap().lights(all_orders, elevator_clone.clone());
            }

            known_elevators_locked.get_mut(0).unwrap().print_status();
            drop(known_elevators_locked);

        }
    });

    // ------------------ MAIN LOOP ---------------------
    loop {
        cbc::select! {
            
            recv(io_channels.light_update_rx) -> a => {
                //Turn onn all lights in own queue
                let mut known_elevators_locked = system_state.known_elevators.lock().unwrap();
                let all_orders = system_state.all_orders.lock().unwrap().clone();
                known_elevators_locked.get_mut(0).unwrap().lights(all_orders, elevator.clone());
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
                        drop(known_elevators_locked);
                        let known_elevators_clone = system_state.known_elevators.lock().unwrap().clone();
                        for elevator in known_elevators_clone.iter(){
                                let success = udphandler.send(&elevator.inn_address, &ordercomplete);
                                udphandler.send(&elevator.inn_address, &msg);
                                if !success {handle_order_completed(&msg,
                                    Arc::clone(&system_state),
                                    io_channels.order_update_tx.clone(), 
                                );

                                }
                        }

                        let mut known_elevators_locked = system_state.known_elevators.lock().unwrap();
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
                    let known_elevators_locked = system_state.known_elevators.lock().unwrap().clone();
                        for elevator in known_elevators_locked.iter(){
                        
                            let send_successfull = udphandler.send(&elevator.inn_address, &msg);

                            if !send_successfull {handle_new_request(&msg,
                                                                     Arc::clone(&system_state),
                                                                     Arc::clone(&udphandler), 
                                                                     io_channels.order_update_tx.clone(), 
                                                                     io_channels.light_update_tx.clone());
                                                }
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
                if known_elevators_locked.get_mut(0).unwrap().queue.is_empty(){
                    elevator.motor_direction(DIRN_STOP);
                }
                known_elevators_locked.get_mut(0).unwrap().go_next_floor(io_channels.door_tx.clone(),io_channels.obstruction_rx.clone(),elevator.clone());
                let all_orders = system_state.all_orders.lock().unwrap().clone();
                known_elevators_locked.get_mut(0).unwrap().lights(all_orders, elevator.clone());
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
                        known_elevators_locked.get_mut(0).unwrap().set_status(Status::Stop, elevator.clone());
                        drop(known_elevators_locked);
                        let system_state_clone = Arc::clone(&system_state);
                        send_new_online(&system_state_clone);

                    }else{
                        known_elevators_locked.get_mut(0).unwrap().set_status(Status::Stop, elevator.clone());
                        
                        //WHO CONTROLS THE LIGHTS
                        known_elevators_locked.get_mut(0).unwrap().turn_off_lights(elevator.clone());
                        drop(known_elevators_locked);
                        let system_state_clone = Arc::clone(&system_state);
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
                        let all_orders = system_state.all_orders.lock().unwrap().clone();
                        known_elevators_locked.get_mut(0).unwrap().lights(all_orders, elevator.clone());
                    }
                    drop(known_elevators_locked);
                }
            },
        }
    }
}
