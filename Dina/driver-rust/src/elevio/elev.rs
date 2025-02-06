#![allow(dead_code)]

use std::fmt;
use std::io::*;
use std::net::TcpStream;
use std::sync::*;


#[derive(Clone, Debug)]
pub struct Elevator {
    socket: Arc<Mutex<TcpStream>>,
    pub num_floors: u8,
    pub ID:i8,
    pub current_floor:i8,
    pub going_up:bool,
    pub queue:Vec<i8>,
    pub status:Status,
    pub direction:i8
}

#[derive(Clone, Debug)]
pub enum Status{
    Idle,
    Moving,
    Maintenance,
    Error
}

pub const HALL_UP: u8 = 0;
pub const HALL_DOWN: u8 = 1;
pub const CAB: u8 = 2;

pub const DIRN_DOWN: u8 = u8::MAX;
pub const DIRN_STOP: u8 = 0;
pub const DIRN_UP: u8 = 1;

impl Elevator {
    pub fn init(addr: &str, num_floors: u8) -> Result<Elevator> {
        Ok(Self {
            socket: Arc::new(Mutex::new(TcpStream::connect(addr)?)),
            num_floors,
            ID: 0,
            current_floor: 0,
            going_up: true,
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

    pub fn add_to_queue(&mut self, floor: i8) {
        if !self.queue.contains(&floor) {
            self.queue.push(floor);
            self.sort_queue();
        }
        else{
            self.print_status();
        }
    }
    

    // Sets current status (Enum Status) for elevator,
    pub fn set_status(&mut self, status: Status){
        match status{

            Status::Maintenance => {
                self.status = Status::Maintenance;
                self.queue.clear();
            }

            // Floors are read as i8, + is going up, - is going down.
            Status::Moving => {
                let first_item_in_queue = self.queue.first().unwrap();
                if *first_item_in_queue < self.current_floor {
                    self.direction = -1;
                    self.current_floor = *first_item_in_queue;
                    self.queue.remove(0);
                } else{
                    self.direction = 1;
                    self.current_floor = *first_item_in_queue;
                    self.queue.remove(0);
                }
            }

            Status::Idle => {
                self.status = Status::Idle;
                self.direction = 0;
            }

            Status::Error => {
                self.status = Status::Error;
                self.queue.clear();
                self.print_status();
            }
        }
    pub fn sort_queue(&mut self) -> Vec<i8> {
        let (mut non_negative, mut negative): (Vec<i8>, Vec<i8>) = self.queue
            .into_iter()
            .partition(|&x| x >= 0);
    
        non_negative.sort();
        negative.sort();
    
        // Non-negative numbers first, negative numbers last
        non_negative.extend(negative);

        let (mut infront, mut behind): (Vec<i8>, Vec<i8>) = non_negative
        .into_iter()
        .partition(|&x| x <= self.floor);

        infront.extend(behind);
        return infront;
    }


    // Moves to next floor, if empty queue, set status to idle.
    pub fn go_next_floor(&mut self) {
        if let Some(next_floor) = self.queue.first() {
            if *next_floor > self.current_floor {
                self.direction = Some(1 as i8);
                self.current_floor += 1;
                self.set_status(Status::Moving);
            } else if *next_floor < self.current_floor {
                self.direction = Some(-1 as i8);
                self.current_floor -= 1;
                self.set_status(Status::Moving);
            } else {
                self.direction = None;
            }
        } else {
            self.set_status(Status::Idle);
        }
    }

    fn print_status(&mut self){
        println!("status: {}",self.status);
    }
    }
}


impl fmt::Display for Elevator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let addr = self.socket.lock().unwrap().peer_addr().unwrap();
        write!(f, "Elevator@{}({})", addr, self.num_floors)
    }
}
