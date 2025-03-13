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

#[derive(Clone, Debug, PartialEq)]
pub enum Status{
    Idle,
    Moving,
    DoorOpen,
    Error,
    Stop
}

impl Status{
    pub fn as_str(&self) -> &str{
        match self{
            Status::Idle => "Idle",
            Status::Moving => "Moving",
            Status::DoorOpen => "DoorOpen",
            Status::Error => "Error",
            Status::Stop => "Stop"
        }
        
    }
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

    pub fn add_to_queue(&mut self, floor: u8) {
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

            // Floors are read as u8 0 is hall up, 1 hall down, 2 cab
            Status::Moving => {
                //HVIS DET ER EN ERROR MÅ VI SE OM DET VAR FORRIGE STATUS DA SKAL VI IKKE GJØRE NOE
                match self.status{
                
                    Status::Moving | Status::Idle => {
                        self.status = Status::Moving;
                        let first_item_in_queue = self.queue.first().unwrap();
                        if *first_item_in_queue < self.current_floor {
                            self.direction = -1;
                            
                        } else if *first_item_in_queue > self.current_floor{
                            self.direction = 1;
                        }
                    }

                    Status::Stop =>{
                        self.status = Status::Stop;
                        
                    }
                    _ =>{
                        //Do Something? 
                    }

                }
                //IMPLEMENT LIGHT FUNCTIONALITY HERE

            }

            Status::DoorOpen=> {
                match self.status{
                    Status::DoorOpen => {
                        self.status = Status::Idle;
                    }
                    _ => {
                        self.status = Status::DoorOpen;
                    }
                }
                
            }

            Status::Idle => {
                match self.status{
                    Status::Stop =>{
                        self.status = Status::Stop;
                        //Do Something? 
                    }
                    _ => {
                        self.status = Status::Idle;

                        //SKRUR AV LYSET FOR DER DEN ER
                        if self.direction == -1{
                            self.call_button_light(self.current_floor, HALL_UP , false);
                        }else{
                            self.call_button_light(self.current_floor, HALL_DOWN , false);
                        };
                        self.call_button_light(self.current_floor, CAB , false);

                        //SIER DEN IKKE BEVEGER SEG LENGER
                        self.direction = 0;
                    }
                }
                
                
            }

            //From stop you can only swap out by calling stop again
            Status::Stop => {
                match self.status{
                    Status::Stop => {
                        self.status = Status::Idle;
                    }
                    _ => {
                        // KILL ELEVATOR !?
                        for f in 0..(self.num_floors) {
                            for c in 0..3 {
                                self.call_button_light(f, c, false);
                            }
                        }

                        self.motor_direction(DIRN_STOP);
                        self.status = Status::Stop;
                        self.queue.clear();
                        self.print_status();
                    }
                } 
            }

            Status::Error => {
                match self.status{
                    Status::Error =>{
                        self.status = Status::Idle;
                    }
                    _ =>{

                        // KILL ELEVATOR !

                        self.motor_direction(DIRN_STOP);
                        self.status = Status::Error;
                        self.queue.clear();
                        self.print_status();
                        /*
                        let msg: Vec<u8> = "ded"
                        make_Udp_msg(self, Error_offline, msg);
                        */
                    }
                }
                
            }
        }
    }
    pub fn sort_queue(&self) -> Vec<u8> {
        let mut sorted_queue = self.queue.clone();
        let (mut non_negative, mut negative): (Vec<u8>, Vec<u8>) = sorted_queue
            .into_iter()
            .partition(|&x| x >= 0);
    
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

    fn print_status(&self){
        println!("status:{}", self.status.as_str());
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



