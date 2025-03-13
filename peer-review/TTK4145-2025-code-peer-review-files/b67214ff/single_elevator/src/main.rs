#![allow(unused)]

use std::env;
use single_elevator::interface::network_unit::ElevatorUDP;
use std::{thread, time};

fn main() {
    //Terminal commands to run on a single Linux unit with communication between elevators. For single elevator 
    //execution, comment theseout and uncomment the "hard coded" ones below
    
    //cargo run -- 0 localhost:12345 19735 19736 19738 19739
    //cargo run -- 1 localhost:12346 19736 19735 19739 19738
    //Note, remember to change N_ELEVATORS
    let args: Vec<String> = env::args().collect();
    let elevator_id = args[1].parse().unwrap();
    let elevator_port = args[2].clone();
    let bcast_port = args[3].parse().unwrap();
    let receive_port = args[4].parse().unwrap();
    let peer_listen_port: u16  = args[5].parse().unwrap();
    let peer_send_port: u16  = args[6].parse().unwrap();

    // let elevator_id = 0;
    // let elevator_port = "localhost:15657".to_string();
    // let bcast_port: u16 = 19735; //For simplified testing work flow
    // let receive_port: u16 = 19736; //For simplified testing work flow
    let elevator_udp = ElevatorUDP::set_up_channels_and_run_all(elevator_id, elevator_port, bcast_port, receive_port, peer_listen_port, peer_send_port);
    loop{
    }
}