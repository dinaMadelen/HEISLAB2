// public crates
use std::{
    net::{SocketAddr, IpAddr, Ipv4Addr},
    thread::*,
    sync::Arc
};

// project crates
use crate::modules::{
    io::io_init::IoChannels, 
    udp_functions::udp::*,
    system_status::SystemState
};

/// Generates a UDP message contaiting a empty worldview
pub fn create_empty_worldview_msg() -> UdpMsg {
    UdpMsg {
        header: UdpHeader {
            sender_id: 0,
            message_type: MessageType::Worldview,
            checksum: 0,
        },
        data: UdpData::Checksum(0),
    }
}

/// Creates a socket address with a base port number + a offset
pub fn create_socket_address<T>(base_port: u16, port_offset: T) -> SocketAddr
where
    T: Into<u16>,
{
    let port_offset: u16 = port_offset.into();
    let port: u16 = base_port + port_offset;

    SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), port)
}

/// Initializes udp reciever
pub fn spawn_udp_reciever_thread(udphandler_clone: Arc<UdpHandler>, system_state_clone: Arc<SystemState>, io_channels_clone: IoChannels) {
    spawn(move||{
        loop{
            let handler = udphandler_clone.clone(); 
            handler.receive(60000, &system_state_clone, io_channels_clone.order_update_tx.clone(), io_channels_clone.light_update_tx.clone());
        }
    });
}

pub fn broadcast_alive_msg(udphandler_clone: Arc<UdpHandler>, system_state_clone: Arc<SystemState>) -> () {
    // get known elevators
    let  known_elevators_locked = system_state_clone.known_elevators.lock().unwrap();
    // get cab
    let cab_clone = known_elevators_locked.get(0).unwrap().clone();

    // create message 
    let msg = make_udp_msg(system_state_clone.me_id, MessageType::NewOnline, UdpData::Cab(cab_clone));
    // define port range
    let start_port: u16 = 3700;
    let end_port: u16 = 3799;
    let port_range = start_port..end_port;
    // broadcast message at specified port range
    for port in port_range{
        let inn_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),port as u16);
        udphandler_clone.send(&inn_addr, &msg);
    }
}
