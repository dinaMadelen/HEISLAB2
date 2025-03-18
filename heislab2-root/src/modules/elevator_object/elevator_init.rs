#![allow(dead_code)]
#![warn(unused_variables)]
#[allow(unused_imports)]

use std::io::*;
use std::fmt;
use std::io::*;
use std::sync::*;
use std::time::Duration;
use std::thread;
use std::convert::TryInto;
use std::net::{TcpStream,IpAddr, Ipv4Addr, SocketAddr,UdpSocket}; // https://doc.rust-lang.org/std/net/enum.IpAddr.html
use std::sync::{Arc, Mutex};
use std::io::ErrorKind;
use serde::{Deserialize, Serialize};
use local_ip_address::local_ip;

pub use crate::modules::system_status::SystemState;
pub use crate::modules::elevator_object::*;
pub use crate::modules::master::Role;
pub use super::elevator_status_functions::Status;
pub use crate::modules::order_object::order_init::Order;
pub use super::alias_lib::{HALL_DOWN, HALL_UP,CAB, DIRN_DOWN, DIRN_UP, DIRN_STOP};


//-------------- GLOBALS/and CONSTANTS


#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Elevator {

   // #[serde(skip)] // TcpStream cant be serialized, do we need it to be TcpSteam?, i dont see what we need TCPstream for as we use UDP
   //pub socket: Arc<Mutex<TcpStream>>,

    pub inn_address: SocketAddr,  // UDP Adress for reciving messages
    pub out_address: SocketAddr,  // UDP Adress for sending messages
    pub num_floors: u8,           // Isnt this the same for every elevator
    pub ID: u8,                   // ID for this spesific elevaotr
    pub current_floor: u8,        // Which floor the elevator was last registerd at      
    pub queue: Vec<Order>,        // The current queue the elevator is servicing
    pub status: Status,           // Current status of the elevator
    pub direction: i8,            // Current direction the elevator is headed
    pub role: Role,               // Current Role of this elevator
}


impl Elevator {
  
    pub fn init(inn_addr: &SocketAddr, out_addr: &SocketAddr, num_floors: u8, id: u8,state:&mut SystemState) -> std::io::Result<Elevator> {
        let my_id = 0; // Should we make a config file for each computer?
        let inport = 3500;
        let outport = 3600;

        let (inn, out) = if id == state.me_ID {
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


        return Ok(Elevator{
                //socket: Arc::new(Mutex::new(TcpStream::connect(addr)?)),
                inn_address: inn,
                out_address: out,
                num_floors,
                ID: id,
                current_floor: 1,
                queue: Vec::new(),
                status: Status::Idle,
                direction: 0,
                role: Role::Slave,
            });
    }



    pub fn motor_direction(&self, dirn: u8) {
        /* 
        let buf = [1, dirn, 0, 0];
        let mut sock = self.socket.lock().unwrap();
        sock.write(&buf).unwrap();
        */
    }

    pub fn call_button_light(&self, floor: u8, call: u8, on: bool) {
        /* 
        let buf = [2, call, floor, on as u8];
        let mut sock = self.socket.lock().unwrap();
        sock.write(&buf).unwrap();
        */
    }

    pub fn floor_indicator(&self, floor: u8) {
        /* 
        let buf = [3, floor, 0, 0];
        let mut sock = self.socket.lock().unwrap();
        sock.write(&buf).unwrap();
        */
    }

    pub fn door_light(&self, on: bool) {
        /* 
        let buf = [4, on as u8, 0, 0];
        let mut sock = self.socket.lock().unwrap();
        sock.write(&buf).unwrap();
        */
    }

    pub fn stop_button_light(&self, on: bool) {
        /* 
        let buf = [5, on as u8, 0, 0];
        let mut sock = self.socket.lock().unwrap();
        sock.write(&buf).unwrap();
        */
    }

    pub fn call_button(&self, floor: u8, call: u8) -> bool {
        return true;
        /* 
        let mut buf = [6, call, floor, 0];
        let mut sock = self.socket.lock().unwrap();
        sock.write(&mut buf).unwrap();
        sock.read(&mut buf).unwrap();
        buf[1] != 0
        */
    }

    pub fn floor_sensor(&self) -> Option<u8> {
        return Some(1);
        /* 
        let mut buf = [7, 0, 0, 0];
        let mut sock = self.socket.lock().unwrap();
        sock.write(&buf).unwrap();
        sock.read(&mut buf).unwrap();
        if buf[1] != 0 {
            Some(buf[2])
        } else {
            None
        }
        */
    }

    pub fn stop_button(&self) -> bool {
        return true;
        /* 
        let mut buf = [8, 0, 0, 0];
        let mut sock = self.socket.lock().unwrap();
        sock.write(&buf).unwrap();
        sock.read(&mut buf).unwrap();
        buf[1] != 0
        */
    }

    pub fn obstruction(&self) -> bool {
        return true;
        /* 
        let mut buf = [9, 0, 0, 0];
        let mut sock = self.socket.lock().unwrap();
        sock.write(&buf).unwrap();
        sock.read(&mut buf).unwrap();
        buf[1] != 0
        */
    }

}

impl fmt::Display for Elevator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,"Elevator@{} (ID: {}, Status: {:?}, Current Floor: {})",self.inn_address,self.ID,self.status,self.current_floor)
    }
}


//Made som changes to the struct and init, Originals are kept in the comments, see below.
/* 
#[derive(Clone, Debug)]
pub struct Elevator {
    socket: Arc<Mutex<TcpStream>>,
    pub num_floors: u8,
    pub ID: u8,
    pub current_floor:u8,
    pub queue:Vec<Order>,
    pub status:Status,
    pub direction:i8,
    pub role:Role,
}



impl Elevator {
    
    pub fn init(addr: &str, num_floors: u8) -> Result<Elevator> {
        Ok(Self {
            socket: Arc::new(Mutex::new(TcpStream::connect(addr)?)),
            num_floors,
            ID: 0,
            current_floor: 1,
            queue: Vec::new(),
            status: Status::Idle,
            direction: 0,
            role: Role::Slave,
        })
    }
    



        Elevator{
            //socket: Arc::new(Mutex::new(TcpStream::connect(addr)?)),
            inn_address: inn,
            out_address: out,
            num_floors,
            ID: id,
            current_floor: 1,
            queue: Vec::new(),
            status: Status::Idle,
            direction: 0,
            role: Role::Slave,
        }
    }



    pub fn motor_direction(&self, dirn: u8) {
        let buf = [1, dirn, 0, 0];
        let mut sock = self.socket.lock().unwrap();
        sock.write(&buf).unwrap();
    }

    pub fn call_button_light(&self, floor: u8, call: u8, on: bool) {
        let buf = [2, call, floor, on as u8];
        let mut sock = self.socket.lock().unwrap();
        sock.write(&buf).unwrap();
    }

    pub fn floor_indicator(&self, floor: u8) {
        let buf = [3, floor, 0, 0];
        let mut sock = self.socket.lock().unwrap();
        sock.write(&buf).unwrap();
    }

    pub fn door_light(&self, on: bool) {
        let buf = [4, on as u8, 0, 0];
        let mut sock = self.socket.lock().unwrap();
        sock.write(&buf).unwrap();
    }

    pub fn stop_button_light(&self, on: bool) {
        let buf = [5, on as u8, 0, 0];
        let mut sock = self.socket.lock().unwrap();
        sock.write(&buf).unwrap();
    }

    pub fn call_button(&self, floor: u8, call: u8) -> bool {
        let mut buf = [6, call, floor, 0];
        let mut sock = self.socket.lock().unwrap();
        sock.write(&mut buf).unwrap();
        sock.read(&mut buf).unwrap();
        buf[1] != 0
    }

    pub fn floor_sensor(&self) -> Option<u8> {
        let mut buf = [7, 0, 0, 0];
        let mut sock = self.socket.lock().unwrap();
        sock.write(&buf).unwrap();
        sock.read(&mut buf).unwrap();
        if buf[1] != 0 {
            Some(buf[2])
        } else {
            None
        }
    }

    pub fn stop_button(&self) -> bool {
        let mut buf = [8, 0, 0, 0];
        let mut sock = self.socket.lock().unwrap();
        sock.write(&buf).unwrap();
        sock.read(&mut buf).unwrap();
        buf[1] != 0
    }

    pub fn obstruction(&self) -> bool {
        let mut buf = [9, 0, 0, 0];
        let mut sock = self.socket.lock().unwrap();
        sock.write(&buf).unwrap();
        sock.read(&mut buf).unwrap();
        buf[1] != 0
    }

}

impl fmt::Display for Elevator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let addr = self.socket.lock().unwrap().peer_addr().unwrap();
        write!(f, "Elevator@{}({})", addr, self.num_floors)
    }
}
*/