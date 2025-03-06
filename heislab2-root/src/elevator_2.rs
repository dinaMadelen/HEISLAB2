#![allow(dead_code)]

use std::convert::TryInto;
use std::fmt;
use std::io::*;
use std::net::TcpStream;
use std::sync::*;
use std::thread;
use std::time::Duration;

#[derive(Clone, Debug, PartialEq)]
pub enum ElevatorStatus {
    Idle,
    Moving,
    DoorOpen,
    Error,
    Stop,
    MovingUp,   //kristoffer
    MovingDown, //kristoffer
}

impl ElevatorStatus {
    pub fn as_str(&self) -> &str {
        match self {
            ElevatorStatus::Idle => "Idle",
            ElevatorStatus::Moving => "Moving",
            ElevatorStatus::DoorOpen => "DoorOpen",
            ElevatorStatus::Error => "Error",
            ElevatorStatus::Stop => "Stop",
            ElevatorStatus::MovingUp => "MovingUp", //kristoffer
            ElevatorStatus::MovingDown => "MovingDown", //kristoffer
        }
    }
}

//------------
// Kristoffer
//------------
#[derive(Clone, Debug, PartialEq)]
pub enum CallType {
    PendingUp,
    PendingDown,
    ServingUp,
    ServingDown,
}

impl CallType {
    pub fn as_str(&self) -> &str {
        match self {
            CallType::PendingUp => "PendingUp",
            CallType::PendingDown => "PendingDown",
            CallType::ServingUp => "ServingUp",
            CallType::ServingDown => "ServingDown",
        }
    }
}
//------------

#[derive(Clone, Debug)]
pub struct Elevator {
    socket: Arc<Mutex<TcpStream>>,
    pub num_floors: u8,
    pub ID: u8,
    pub current_floor: u8,
    pub queue: Vec<u8>,
    pub status: ElevatorStatus,
    pub direction: i8,
}

pub const HALL_UP: u8 = 0;
pub const HALL_DOWN: u8 = 1;
pub const CAB: u8 = 2;

pub const DIRN_DOWN: u8 = u8::MAX;
pub const DIRN_STOP: u8 = 0;
pub const DIRN_UP: u8 = 1;

impl Elevator {
    /// Initializes a instance of the elevator struct (creates a elevator)
    pub fn init(addr: &str, num_floors: u8) -> Result<Elevator> {
        Ok(Self {
            socket: Arc::new(Mutex::new(TcpStream::connect(addr)?)),
            num_floors,
            ID: 0,
            current_floor: 1,
            queue: Vec::new(),
            status: ElevatorStatus::Idle,
            direction: 0,
        })
    }

    /// Sets the motor direction
    pub fn motor_direction(&self, dirn: u8) {
        let buf = [1, dirn, 0, 0];
        let mut sock = self.socket.lock().unwrap();
        sock.write(&buf).unwrap();
    }

    /// Turns on call buttn light at speficic floor
    pub fn call_button_light(&self, floor: u8, call: u8, on: bool) {
        let buf = [2, call, floor, on as u8];
        let mut sock = self.socket.lock().unwrap();
        sock.write(&buf).unwrap();
    }

    /// Indicates floor
    pub fn floor_indicator(&self, floor: u8) {
        let buf = [3, floor, 0, 0];
        let mut sock = self.socket.lock().unwrap();
        sock.write(&buf).unwrap();
    }

    /// Sets door light
    pub fn door_light(&self, on: bool) {
        let buf = [4, on as u8, 0, 0];
        let mut sock = self.socket.lock().unwrap();
        sock.write(&buf).unwrap();
    }

    /// Sets stop button light
    pub fn stop_button_light(&self, on: bool) {
        let buf = [5, on as u8, 0, 0];
        let mut sock = self.socket.lock().unwrap();
        sock.write(&buf).unwrap();
    }

    /// Sets a call to a given floor
    pub fn call_button(&self, floor: u8, call: u8) -> bool {
        let mut buf = [6, call, floor, 0];
        let mut sock = self.socket.lock().unwrap();
        sock.write(&mut buf).unwrap();
        sock.read(&mut buf).unwrap();
        buf[1] != 0
    }

    /// Queries the elevator system for the current floor, and returns `Some(<floor>)` with the floor number if the system responds to the query.
    /// If the floor is not known, the function returns `None`      
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

    /// Sets the stop button
    pub fn stop_button(&self) -> bool {
        let mut buf = [8, 0, 0, 0];
        let mut sock = self.socket.lock().unwrap();
        sock.write(&buf).unwrap();
        sock.read(&mut buf).unwrap();
        buf[1] != 0
    }

    /// Sets obstruction
    pub fn obstruction(&self) -> bool {
        let mut buf = [9, 0, 0, 0];
        let mut sock = self.socket.lock().unwrap();
        sock.write(&buf).unwrap();
        sock.read(&mut buf).unwrap();
        buf[1] != 0
    }

    /// Add floor to queue
    pub fn add_to_queue(&mut self, floor: u8) {
        if !self.queue.contains(&floor) {
            self.queue.push(floor);
            self.sort_queue();
        } else {
            self.print_status();
        }
    }

    /// Sets status for elevator
    pub fn set_status(&mut self, status: ElevatorStatus) {
        match status {
            ElevatorStatus::Moving   => self.handle_moving_status(),
            ElevatorStatus::DoorOpen => self.handle_door_open_status(),
            ElevatorStatus::Idle     => self.handle_idle_status(),
            ElevatorStatus::Stop     => self.handle_stop_status(),
            ElevatorStatus::Error    => self.handle_error_status(),
            _                        => (), // denne må fikses
        }
    }

    pub fn handle_moving_status(&mut self) -> () {
        //HVIS DET ER EN ERROR MÅ VI SE OM DET VAR FORRIGE STATUS DA SKAL VI IKKE GJØRE NOE
        match self.status {
            ElevatorStatus::Moving | ElevatorStatus::Idle => {
                self.status = ElevatorStatus::Moving;
                let first_item_in_queue = self.queue.first().unwrap();
                if *first_item_in_queue < self.current_floor {
                    self.direction = -1;
                } else if *first_item_in_queue > self.current_floor {
                    self.direction = 1;
                }
            }
            _ => {
                //Do Something?
            }
        }
        //IMPLEMENT LIGHT FUNCTIONALITY HERE
    }

    pub fn handle_door_open_status(&mut self) {
        match self.status {
            ElevatorStatus::DoorOpen => {
                self.status = ElevatorStatus::Idle;
            }
            _ => {
                self.status = ElevatorStatus::DoorOpen;
            }
        }
    }

    pub fn handle_idle_status(&mut self) {
        self.status = ElevatorStatus::Idle;

        //SKRUR AV LYSET FOR DER DEN ER
        if self.direction == -1 {
            self.call_button_light(self.current_floor, HALL_UP, false);
        } else {
            self.call_button_light(self.current_floor, HALL_DOWN, false);
        };
        self.call_button_light(self.current_floor, CAB, false);

        //SIER DEN IKKE BEVEGER SEG LENGER
        self.direction = 0;
    }

    pub fn handle_stop_status(&mut self) {
        match self.status {
            ElevatorStatus::Stop => {
                self.status = ElevatorStatus::Idle;
            }
            _ => {
                // KILL ELEVATOR !?
                for f in 0..(self.num_floors) {
                    for c in 0..3 {
                        self.call_button_light(f, c, false);
                    }
                }

                self.motor_direction(DIRN_STOP);
                self.status = ElevatorStatus::Stop;
                self.queue.clear();
                self.print_status();
            }
        }
    }

    pub fn handle_error_status(&mut self) {
        match self.status {
            ElevatorStatus::Error => {
                self.status = ElevatorStatus::Idle;
            }
            _ => {
                // KILL ELEVATOR !

                self.motor_direction(DIRN_STOP);
                self.status = ElevatorStatus::Error;
                self.queue.clear();
                self.print_status();
            }
        }
    }

    pub fn sort_queue(&self) -> Vec<u8> {
        let mut sorted_queue = self.queue.clone();
        let (mut non_negative, mut negative): (Vec<u8>, Vec<u8>) =
            sorted_queue.into_iter().partition(|&x| x >= 0);

        non_negative.sort();
        negative.sort();

        // Non-negative numbers first, negative numbers last
        non_negative.extend(negative);

        let (mut infront, mut behind): (Vec<u8>, Vec<u8>) = non_negative
            .into_iter()
            .partition(|&x| x <= self.current_floor);

        infront.extend(behind);
        return infront;
    }

    // Moves to next floor, if empty queue, set status to idle. If !(moving  idle), do nothing
    pub fn go_next_floor(&mut self) {
        if ((self.status == ElevatorStatus::Moving) | (self.status == ElevatorStatus::Idle)) {
            if let Some(next_floor) = self.queue.first() {
                if *next_floor > self.current_floor {
                    self.set_status(ElevatorStatus::Moving);
                    self.motor_direction(DIRN_UP);
                    //self.current_floor += 1;
                } else if *next_floor < self.current_floor {
                    self.set_status(ElevatorStatus::Moving);
                    self.motor_direction(DIRN_DOWN);
                    //self.current_floor -= 1;
                } else if *next_floor == self.current_floor {
                    self.set_status(ElevatorStatus::Idle);
                    self.motor_direction(DIRN_STOP);
                    self.queue.remove(0);

                    self.door_open_sequence();
                }
            } else {
                self.set_status(ElevatorStatus::Idle);
                self.motor_direction(DIRN_STOP);
            }
        } else {
            self.motor_direction(DIRN_STOP);
        }
    }

    fn print_status(&self) {
        println!("status:{}", self.status.as_str());
    }

    //MIDLERTIDIG FUNKSJON
    pub fn door_open_sequence(&mut self) {
        self.set_status(ElevatorStatus::DoorOpen);

        let handle = thread::spawn(|| {
            thread::sleep(Duration::from_secs(2)); // Sleep for 2 seconds

            println!("Thread woke up!");
        });

        handle.join().unwrap(); // Wait for the thread to finish

        self.set_status(ElevatorStatus::DoorOpen);
        self.go_next_floor();
    }
}

impl fmt::Display for Elevator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let addr = self.socket.lock().unwrap().peer_addr().unwrap();
        write!(f, "Elevator@{}({})", addr, self.num_floors)
    }
}
