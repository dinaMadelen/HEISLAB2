use crossbeam_channel as cbc;
use std::{
    thread::*,
    time::*,
    sync::Arc,
    net::{SocketAddr, IpAddr, Ipv4Addr},
};

use heislab2_root::modules::{
    cab_object::elevator_status_functions::Status,
    order_object::order_init::Order,
    slave_functions::slave::*,
    master_functions::master::*,
    elevator_object::elevator_init::Elevator,
    udp_functions::udp_handler_init::*,
    udp_functions::message_handlers::*,
    udp_functions::udp::*,
    io::io_init::*,
    elevator_object::alias_lib::{DIRN_DOWN, DIRN_STOP},
    system_init::*,
    cab_object::cab::Cab,
    monitoring_threads::*,
};
use local_ip_address::local_ip;

fn main() -> std::io::Result<()> {

    //--------------INIT ELEVATOR------------

    // Check boot function in system_Init.rs
    let elev_num_floors = 4;
    // let elevator = Elevator::init("localhost:15000", elev_num_floors)?;
   
    let elevator = Elevator::init("localhost:15659", elev_num_floors)?;

    println!("Elevator started:\n{:#?}", elevator);

    //--------------INIT ELEVATOR FINISH------------

    // --------------INIT CAB---------------
    let system_state = Arc::new(boot());
    
    let inn_addr = SocketAddr::new(local_ip().unwrap(), 3700 + system_state.me_id as u16);
    let out_addr = SocketAddr::new(local_ip().unwrap(), 3800 + system_state.me_id as u16);
    
    let set_id = system_state.me_id;
    println!("me id is {}",system_state.me_id);
   
    let mut cab = Cab::init(&inn_addr, &out_addr, elev_num_floors, set_id, &system_state)?;
    cab.turn_off_lights(elevator.clone());

    //---------------INIT UDP HANDLER-------------------
    let udphandler = Arc::new(init_udp_handler(cab.clone()));
    //-------------INIT UDP HANDLER FINISH-----------------

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
    
    let new_online_msg = make_udp_msg(system_state.me_id, MessageType::NewOnline, UdpData::Cab(cab_clone));
    let known_elevators_locked = system_state.known_elevators.lock().unwrap();
    udp_broadcast(&new_online_msg);

   

    /* ---- -- - ----- -----INIT ELEVATOR MONITOR - Can be found in monitoring_threads ---- - --------- */
    let system_state_clone = Arc::clone(&system_state);
    let udp_handler_clone = Arc::clone(&udphandler);


    spawn_master_monitor(system_state_clone, 
                        udp_handler_clone);


     /* ---- -- - ------ -----INIT QUEUE FINISHER - Can be found in monitoring_threads ---- - --------- */
    let system_state_clone = Arc::clone(&system_state);
    let elevator_clone = elevator.clone();
    let door_tx_clone = io_channels.door_tx.clone();
    let obstruction_rx_clone = io_channels.obstruction_rx.clone();


    spawn_queue_finisher(elevator_clone.clone(),
                system_state_clone,
                door_tx_clone.clone(),
                obstruction_rx_clone.clone());



    // ------------------ MAIN LOOP ---------------------
    loop {
        cbc::select! {
            /* ------- --- -- NEW LIGHT UPDATE  -- ----  ------*/
            recv(io_channels.light_update_rx) -> a => {
                //Turn onn all lights in own queue
                let mut known_elevators_locked = system_state.known_elevators.lock().unwrap().clone();
                known_elevators_locked.get_mut(0).unwrap().lights(&system_state.clone(), elevator.clone());
            },


            /* ------- --- -- NEW ORDER UPDATE  -- ----  ------*/
            recv(io_channels.order_update_rx) -> a => {
                let mut known_elevators_locked = system_state.known_elevators.lock().unwrap();
                if !known_elevators_locked.is_empty() {

                    println!("Current queue: {:?}",known_elevators_locked.get_mut(0).unwrap().queue);

                    /*      GET A SENDABLE CLONE OF CAB     */
                    let cab_clone = known_elevators_locked.get(0).unwrap().clone();

                    /* IF ELEVATOR STATUS IDLE SEND AN "IM ALIVE" MESSAGE TO SYSTEM TO UPDATE SYSTEM OF CURRENT STATE */
                    if known_elevators_locked.get_mut(0).unwrap().status == Status::Idle 
                    {
                        let imalive = make_udp_msg(system_state.me_id,
                                                MessageType::ImAlive,
                                                UdpData::Cab(cab_clone));

                        for elevator in known_elevators_locked.iter()
                        {
                            udphandler.send(&elevator.inn_address, &imalive);
                        }
                    }

                    known_elevators_locked.get_mut(0).unwrap().go_next_floor    (io_channels.door_tx.clone(),
                                                                                io_channels.obstruction_rx.clone(),
                                                                                elevator.clone());
                }

            },
            
            /* ------- --- -- NEW DOOR UPDATE  -- ----  ------*/
            recv(io_channels.door_rx) -> a => {
                /* Retrieve signal */
                let door_closed = a.unwrap();
                
                /* If door is open do nothing*/
                if door_closed {

                    elevator.door_light(false);
                    /* Retrieve known elevators l*/
                    let mut known_elevators_locked = system_state.known_elevators.lock().unwrap().clone();

                    if known_elevators_locked.get_mut(0).unwrap().queue.is_empty(){
                        
                    }else {
                        let mut known_elevators_locked = system_state.known_elevators.lock().unwrap();
                        let cab_clone = known_elevators_locked.get(0).unwrap().clone();

                        known_elevators_locked.get_mut(0).unwrap().set_status(Status::Idle, elevator.clone());
                        let completed_order = known_elevators_locked.get_mut(0).unwrap().queue.remove(0);
                        drop(known_elevators_locked);

                        /*       FIRST REMOVE FROM OWN ALL ORDERS      */
                        let mut all_orders_locked = system_state.all_orders.lock().unwrap();
                        if completed_order.order_type == CAB {
                            if let Some(index) = all_orders_locked.iter().position(|order| (order.floor == completed_order.floor)&& (order.order_type == CAB)) {
                                all_orders_locked.remove(index);
                            }
                        } else {
                            all_orders_locked.retain(|order| {
                                !((order.floor == completed_order.floor )&& (order.order_type == completed_order.order_type))
                            });
                        }  
                        drop(all_orders_locked);
                        // Drop due to long time til next use.

                        let known_elevators_locked = system_state.known_elevators.lock().unwrap();
                        let cab_clone_removed = known_elevators_locked.get(0).unwrap().clone();
                        let alive_msg = make_udp_msg(system_state.me_id, MessageType::ImAlive, UdpData::Cab(cab_clone_removed.clone()));
                        let ordercomplete = make_udp_msg(system_state.me_id, MessageType::OrderComplete, UdpData::Cab(cab_clone.clone()));
                        drop(known_elevators_locked);

                        let elevator_addresses: Vec<_> = {
                            let known_elevators = system_state.known_elevators.lock().unwrap();
                            known_elevators.iter().map(|e| e.inn_address).collect()
                        };

                        for addr in elevator_addresses {
                            udphandler.send(&addr, &ordercomplete);
                            udphandler.send(&addr, &alive_msg); 
                        }

                        elevator.call_button_light(completed_order.floor, completed_order.order_type, false);
                                
                        
                    }
                }                 
            },

            recv(io_channels.call_rx) -> a => {
                let call_button = a.unwrap();
                println!("{:#?}", call_button);
                //Make new order and add that order to elevators queue
                let new_order = Order::init(call_button.floor, call_button.call);
                
                {
                    let mut known_elevators_locked = system_state.known_elevators.lock().unwrap();
                    for elevator in known_elevators_locked.iter_mut(){
                        
                        // add to queue, high priority
                        if (elevator.id == system_state.me_id) && (new_order.order_type == CAB){
                            if elevator.queue.len()>1{
                                elevator.queue.insert(1,new_order.clone());
                            }else {
                                elevator.queue.insert(0,new_order.clone());
                            }
                        }
                    }
                    
                    let new_req_msg = make_udp_msg(system_state.me_id, MessageType::NewRequest, UdpData::Order(new_order.clone()));
                    let known_elevators_clone = known_elevators_locked.clone();
                    drop(known_elevators_locked);
                        for elevator in known_elevators_clone.iter(){
                            

                                    
                            let send_successfull = udphandler.send(&elevator.inn_address, &new_req_msg);

                            if !send_successfull{handle_new_request(&new_req_msg,
                                                                     Arc::clone(&system_state),
                                                                     Arc::clone(&udphandler), 
                                                                     io_channels.order_update_tx.clone(), 
                                                                     io_channels.light_update_tx.clone());
                                                }
                        }
                    
                }

                //cab.turn_on_queue_lights(elevator.clone());
                let mut known_elevators_locked = system_state.known_elevators.lock().unwrap();

                //Safety if elevator is idle to double check if its going to correct floor
                if known_elevators_locked.is_empty(){
                    println!("No active elevators, not even this one ID:{}",system_state.me_id);

                }else if known_elevators_locked.get_mut(0).unwrap().status == Status::Idle{
                    known_elevators_locked.get_mut(0).unwrap().go_next_floor(io_channels.door_tx.clone(),io_channels.obstruction_rx.clone(),elevator.clone());
                    if known_elevators_locked.get_mut(0).unwrap().status == Status::Moving{
                        let alive_msg = make_udp_msg(system_state.me_id, MessageType::ImAlive, UdpData::Cab(known_elevators_locked.get(0).unwrap().clone()));
                        for elevator in known_elevators_locked.iter(){
                            udphandler.send(&elevator.inn_address, &alive_msg);
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

                let mut known_elevators_locked = system_state.known_elevators.lock().unwrap();
                if known_elevators_locked.get_mut(0).unwrap().queue.is_empty(){
                    elevator.motor_direction(DIRN_STOP);
                }

                known_elevators_locked.get_mut(0).unwrap().go_next_floor(io_channels.door_tx.clone(),io_channels.obstruction_rx.clone(),elevator.clone());
                drop(known_elevators_locked);

                let mut known_elevators_clone = system_state.known_elevators.lock().unwrap().clone();
                known_elevators_clone.get_mut(0).unwrap().lights(&system_state.clone(), elevator.clone());
                


                //Broadcast new state
                let  known_elevators_locked = system_state.known_elevators.lock().unwrap();
                let cab_clone = known_elevators_locked.get(0).unwrap().clone();
                let alive_msg = make_udp_msg(system_state.me_id, MessageType::ImAlive, UdpData::Cab(cab_clone));
                    for elevator in known_elevators_locked.iter(){
                        udphandler.send(&elevator.inn_address, &alive_msg);
                       }
                drop(known_elevators_locked);
                
            },

           
            recv(io_channels.stop_rx) -> a => {
                let stop = a.unwrap();
                println!("Stop button: {:#?}", stop);
                let mut known_elevators_locked = system_state.known_elevators.lock().unwrap();
                if known_elevators_locked.is_empty(){
                    println!("There are no elevators in the system");
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
                        send_error_offline(&system_state.clone());
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
                        known_elevators_locked.get_mut(0).unwrap().set_status(Status::Idle, elevator.clone());
                        known_elevators_locked.get_mut(0).unwrap().go_next_floor(io_channels.door_tx.clone(),io_channels.obstruction_rx.clone(),elevator.clone());
                        drop(known_elevators_locked);
                        let mut known_elevators_clone = system_state.known_elevators.lock().unwrap().clone();
                        known_elevators_clone.get_mut(0).unwrap().lights(&system_state.clone(), elevator.clone());
                        
                    }
                    
                }
            },
        }
    }
}
