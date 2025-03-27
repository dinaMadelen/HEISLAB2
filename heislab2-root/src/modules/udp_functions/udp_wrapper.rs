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
use crate::modules::udp_functions::udp::*;


//-----------
// Functions
//-----------
/// ### Description
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

pub fn create_socket_address<T>(base_port: u16, port_offset: T) -> SocketAddr
where
    T: Into<u16>,
{
    let port_offset: u16 = port_offset.into();
    let port: u16 = base_port + port_offset;

    SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), port)
}
