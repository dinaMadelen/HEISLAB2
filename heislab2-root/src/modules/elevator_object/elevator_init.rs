#![allow(dead_code)]
#![warn(unused_variables)]

use std::fmt;
use std::io::*;
use std::net::TcpStream;
use std::sync::*;
use std::time::Duration;
use std::thread;
use std::convert::TryInto;
use modules::alias_lib;


#[derive(Clone, Debug)]
pub struct Elevator {
    socket: Arc<Mutex<TcpStream>>,
    pub num_floors: u8,
    pub ID: u8,
    pub current_floor:u8,
    pub queue:Vec<u8>,
    pub status:Status,
    pub direction:i8
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
        })
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


    // Moves to next floor, if empty queue, set status to idle. If !(moving  idle), do nothing
    pub fn go_next_floor(&mut self) {
        if ((self.status == Status::Moving) | (self.status == Status::Idle)){
            if let Some(next_floor) = self.queue.first() {
                if *next_floor > self.current_floor {
                    self.set_status(Status::Moving);
                    self.motor_direction(DIRN_UP);
                    //self.current_floor += 1;
                    
                } else if *next_floor < self.current_floor {
                    self.set_status(Status::Moving);
                    self.motor_direction(DIRN_DOWN);
                    //self.current_floor -= 1;
                    
                } else if *next_floor == self.current_floor{
                    self.set_status(Status::Idle);
                    self.motor_direction(DIRN_STOP);
                    self.queue.remove(0);
                    
                    self.door_open_sequence();

            
                }
            } else {
                self.set_status(Status::Idle);
                self.motor_direction(DIRN_STOP);
            }
        } else {
            self.motor_direction(DIRN_STOP);
        }
    }


    
    //MIDLERTIDIG FUNKSJON
    pub fn door_open_sequence(&mut self) {
        self.set_status(Status::DoorOpen);

        let handle = thread::spawn(|| {
            thread::sleep(Duration::from_secs(2)); // Sleep for 2 seconds
            
            println!("Thread woke up!");
        });

        //handle.join().unwrap(); // Wait for the thread to finish
        self.set_status(Status::DoorOpen);
        self.go_next_floor();
    }
}



impl fmt::Display for Elevator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let addr = self.socket.lock().unwrap().peer_addr().unwrap();
        write!(f, "Elevator@{}({})", addr, self.num_floors)
    }
}



