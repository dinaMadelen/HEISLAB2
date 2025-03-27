//--------------------
// Module description
//--------------------
//! This module contains functions that wrap different UDP functionality


//---------
// Imports
//---------
// public crates
use std::net::{SocketAddr, IpAddr, Ipv4Addr};

// project crates
use crate::modules::{io::io_init::IoChannels, udp_functions::udp::*};


//-----------
// Functions
//-----------
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
            handler.receive(60000, &system_state_clone, io_channels_clone.order_update_tx, io_channels_clone.light_update_tx);
        }
    });
}