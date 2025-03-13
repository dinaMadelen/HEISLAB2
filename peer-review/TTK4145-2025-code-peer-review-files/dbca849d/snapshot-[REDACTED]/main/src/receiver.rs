use core::net::SocketAddr;
use std::net::UdpSocket;

use crossbeam_channel as cbc;
use log::{debug, info};

use crate::messages;

pub fn run(manager_tx: cbc::Sender<messages::Manager>) {
    debug!("Receiver up and running...");
    let addr: SocketAddr = "0.0.0.0:4567".parse().unwrap();

    let socket = UdpSocket::bind(addr).unwrap();
    info!("Listening on {}", socket.local_addr().unwrap());

    let mut buf = [0u8; 1024];

    loop {
        debug!("Ready for input...");
        let (_, _) = socket.recv_from(&mut buf).unwrap();
        // Deserialize the binary data back to a struct
        let deserialized: messages::Manager = bincode::deserialize(&buf).unwrap();
        manager_tx.send(deserialized).unwrap();
    }
}
