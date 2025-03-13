use network_rust::udpnet;
use std::env;
use std::thread::spawn;
use crossbeam_channel::{self as cbc, select};
use serde::{Serialize, Deserialize};

use crate::state_utils::State;

#[derive(Serialize, Deserialize, Clone)]
pub struct AllEncompassingDataType {
    states: Vec<State>,
    id: String,
    version: u64
}

pub fn udp_main(tx: cbc::Sender<AllEncompassingDataType>, rx: cbc::Receiver<AllEncompassingDataType>) {
    let args: Vec<String> = env::args().collect();
    let id: String = args[1].clone();
    let port: u16 = args[2].parse().unwrap();

    let (custom_data_send_tx, custom_data_send_rx) = cbc::unbounded::<AllEncompassingDataType>();
    spawn(move || udpnet::bcast::tx(port, custom_data_send_rx));

    let (custom_data_recv_tx, custom_data_recv_rx) = cbc::unbounded::<AllEncompassingDataType>();
    spawn(move || udpnet::bcast::rx(port, custom_data_recv_tx));

    loop {
        select! {
            recv(rx) -> data => {
                let received_data = data.clone().unwrap();
                // For debugging
                for s in received_data.states {
                    println!("{:#?}", s);
                }
                //
                custom_data_send_tx.send(data.unwrap()).unwrap();
            },
            recv(custom_data_recv_rx) -> data => {
                let received_data = data.clone().unwrap();
                // For debugging
                for s in received_data.states {
                    println!("{:#?}", s);
                }
                //
                tx.send(data.unwrap()).unwrap();
            }
        }
    }
}