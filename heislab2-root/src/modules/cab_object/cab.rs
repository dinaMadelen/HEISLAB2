#![allow(dead_code)]
#![warn(unused_variables)]
#[allow(unused_imports)]

use std::io::*;
use std::net::{IpAddr, Ipv4Addr, SocketAddr}; // https://doc.rust-lang.org/std/net/enum.IpAddr.html
use serde::{Deserialize, Serialize};
use local_ip_address::local_ip;

pub use crate::modules::system_status::SystemState;
pub use crate::modules::master_functions::master::Role;
pub use super::elevator_status_functions::Status;
pub use crate::modules::order_object::order_init::Order;

pub use crate::modules::elevator_object::*;
pub use elevator_init::Elevator;
pub use alias_lib::{HALL_DOWN, HALL_UP,CAB, DIRN_DOWN, DIRN_UP, DIRN_STOP};


//-------------- GLOBALS/and CONSTANTS


#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Cab {

    pub inn_address: SocketAddr,  // UDP Adress for reciving messages
    pub out_address: SocketAddr,  // UDP Adress for sending messages
    pub num_floors: u8,           // Isnt this the same for every elevator
    pub id: u8,                   // ID for this spesific elevaotr
    pub current_floor: u8,        // Which floor the elevator was last registerd at      
    pub queue: Vec<Order>,        // The current queue the elevator is servicing
    pub status: Status,          // Current status of the elevator
    pub direction: i8,            // Current direction the elevator is headed
    pub role: Role,               // Current Role of this elevator
}


impl Cab {
  
    pub fn init(inn_addr: &SocketAddr, out_addr: &SocketAddr, num_floors: u8, set_id: u8,state:&mut SystemState) -> std::io::Result<Cab> {
        let inport = 3500;
        let outport = 3600;

        let (inn, out) = if set_id == state.me_id {
            match local_ip() {
                Ok(ip) => {
                    let inn = SocketAddr::new(ip, inport);
                    let out = SocketAddr::new(ip, outport);

                    println!("Assigned IP: {} (InPort: {}, OutPort: {})", ip, inport, outport);
                    (inn, out)
                }
                Err(_) => {
                    println!("Could not find local IP address., sets default");
                    let inn = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 3500);
                    let out = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 3600);
                    (inn,out) 
                }
            }
        }else{
            (*inn_addr, *out_addr)
        };


        return Ok(Cab{
                inn_address: inn,
                out_address: out,
                num_floors,
                id: set_id,
                current_floor: 1,
                queue: Vec::new(),
                status: Status::Idle,
                direction: 0,
                role: Role::Slave,
            });
    }
}
