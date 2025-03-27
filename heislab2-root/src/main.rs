use std::{
    thread::*,
    time::*,
    sync::Arc,
    net::{SocketAddr, IpAddr, Ipv4Addr}
};

use crossbeam_channel as cbc;

use heislab2_root::modules::{
    udp_functions::{
        udp::*, 
        udp_wrapper::*
    },
    cab_object::{
        cab_wrapper::*,
        elevator_status_functions::Status,
    }, 
    io::{
        io_init::*,
        io_handlers::*
    },
    system_init::*,
    master_functions::{
        master::*,
        master_wrapper::*
    },
    slave_functions::slave::*,
    elevator_object::{
        alias_lib::DIRN_DOWN,
        elevator_init::Elevator,
        elevator_wrapper::*
    },
    order_object::order_init::Order
};


fn main() -> std::io::Result<()> {
    // initialize elevator
    let elev_num_floors = 4;
    let elev_server_port = 15_657;
    let elevator = initialize_elevator(elev_server_port, elev_num_floors);

    // create dummy empty worldview message 
    // let boot_worldview = udp_wrapper::create_empty_worldview_msg();

    let system_state = initialize_system_state();

    // create socket addresses
    let inn_addr = create_socket_address(3700, system_state.me_id);
    let out_addr = create_socket_address(3800, system_state.me_id);

    // initialize cab and support systems
    let cab = initialize_cab(elev_num_floors, &system_state, elevator.clone(), inn_addr, out_addr)?;
    let udphandler = initialize_udp_handler(cab.clone());
    let io_channels = IoChannels::new(&elevator);

    // initial setup work
    add_cab_to_sys_state(system_state.clone(), cab);
    set_master_id(system_state.clone());
    spawn_udp_reciever_thread(udphandler.clone(), system_state.clone(), io_channels.clone());
    go_down_until_floor_found(&elevator, DIRN_DOWN);
    spawn_elevator_monitor_thread(system_state.clone(), udphandler.clone());
    print_master_id(system_state.clone());
    broadcast_alive_msg(udphandler.clone(), system_state.clone());
    spawn_master_failure_check_thread(system_state.clone(), udphandler.clone());
    spawn_queue_finish_thread(
        system_state.clone(),
        elevator.clone(),
        io_channels.clone()
    );

    // ------------------ MAIN LOOP ---------------------
    loop {
        cbc::select! {
            
            recv(io_channels.light_update_rx) -> _light_update_rx_msg => {
                handle_light_update_rx(system_state.clone(), elevator.clone());
            },
            recv(io_channels.order_update_rx) -> _order_update_rx_msg => {
                handle_order_update_rx(system_state.clone(),udphandler.clone(),elevator.clone(), io_channels.clone());
            },
            recv(io_channels.door_rx) -> door_rx_msg => {
                handle_door_rx(system_state.clone(), udphandler.clone(), io_channels.clone(), door_rx_msg.unwrap(), &elevator);
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
