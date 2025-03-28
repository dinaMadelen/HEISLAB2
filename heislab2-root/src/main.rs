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
        udp_wrapper::*,
        handlers::*,
        reciever::*,
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
    let inn_addr = create_socket_address(3701, system_state.me_id);
    let out_addr = create_socket_address(3801, system_state.me_id);

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

    loop {
        cbc::select! {
            recv(io_channels.light_update_rx) -> _light_update_rx_msg => {
                handle_light_update_rx(system_state.clone(), elevator.clone());
            },
            recv(io_channels.order_update_rx) -> _order_update_rx_msg => {
                handle_order_update_rx(system_state.clone(),udphandler.clone(),elevator.clone(), io_channels.clone());
            },
            recv(io_channels.door_rx) -> door_rx_msg => {
                handle_door_rx(system_state.clone(), udphandler.clone(), door_rx_msg.unwrap(), &elevator);
            },
            recv(io_channels.call_rx) -> call_button_rx_msg => {
                handle_call_rx(call_button_rx_msg.unwrap(), system_state.clone(), udphandler.clone(), io_channels.clone(), &elevator);
            },
            recv(io_channels.floor_rx) -> floor_rx_msg => {
                handle_floor_rx(floor_rx_msg.unwrap(), system_state.clone(), udphandler.clone(), io_channels.clone(), &elevator);
            },
            recv(io_channels.stop_rx) -> stop_rx_msg => {
                handle_stop_rx(stop_rx_msg.unwrap(), system_state.clone(), &elevator);
            },
            recv(io_channels.obstruction_rx) -> obstruction_rx_msg => {
                handle_obstruction_rx(obstruction_rx_msg.unwrap(), system_state.clone(), io_channels.clone(), &elevator);
            },
        }
    }
}
