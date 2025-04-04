#![allow(dead_code)]

use std::fmt;
use std::io::*;
use std::net::TcpStream;
use std::sync::*;
use std::time::Duration;
use std::thread;
use std::convert::TryInto;


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
            Status::Idle     => "Idle",
            Status::Moving   => "Moving",
            Status::DoorOpen => "DoorOpen",
            Status::Error    => "Error",
            Status::Stop     => "Stop"
        }
    }
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
            socket:        Arc::new(Mutex::new(TcpStream::connect(addr)?)),
            num_floors,
            ID:            0,
            current_floor: 1,
            queue:         Vec::new(),
            status:        Status::Idle,
            direction:     0,
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

    pub fn set_status_moving(&mut self){
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

    pub fn set_status_door_open(&mut self){
        match self.status{
            Status::DoorOpen => {
                self.status = Status::Idle;
            }
            _ => {
                self.status = Status::DoorOpen;
            }
        }
    }

    pub fn set_status_idle(&mut self){
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

    pub fn set_status_stop(&mut self){
        //From stop you can only swap out by calling stop again
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

    pub fn set_status_error(&mut self){
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
    
    // Sets current status (Enum Status) for elevator,
    pub fn set_status(&mut self, status: Status){
        match status{
            // Floors are read as u8 0 is hall up, 1 hall down, 2 cab
            Status::Moving   => self.set_status_moving(),
            Status::DoorOpen => self.set_status_door_open(),
            Status::Idle     => self.set_status_idle(),
            Status::Stop     => self.set_status_stop(),
            Status::Error    => self.set_status_error(),
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

    fn shall_not_move(&mut self) -> bool{
        return !((self.status == Status::Moving) || (self.status == Status::Idle));
    }

    fn stop_motor(&mut self){
        self.motor_direction(DIRN_STOP);
    }

    fn allowed_to_move(&mut self) -> bool{
        return !(self.shall_not_move());
    }

    fn queue_is_not_empty(&mut self) -> bool{
        if let Some(next_floor) = self.queue.first() {return true}
        else{return false}
    }

    fn queue_is_empty(&mut self) -> bool{
        return !(self.queue_is_not_empty())
    }

    fn move_up(&mut self){
        self.set_status(Status::Moving);
        self.motor_direction(DIRN_UP);
        //self.current_floor += 1;
    }

    fn move_down(&mut self){
        self.set_status(Status::Moving);
        self.motor_direction(DIRN_DOWN);
        //self.current_floor -= 1;
    }

    fn let_passengers_off(&mut self){
        self.set_status(Status::Idle);
        self.motor_direction(DIRN_STOP);
        self.queue.remove(0);
        self.door_open_sequence();
    }

    fn normal_stop(&mut self){
        self.set_status(Status::Idle);
        self.motor_direction(DIRN_STOP);
    }

    fn going_up(&mut self) -> bool{
        return (self.queue.first() > self.current_floor)
    }

    fn going_down(&mut self) -> bool{
        return (self.queue.first() < self.current_floor)
    }

    fn passenger_off_here(&mut self) -> bool{
        return (self.queue.first() == self.current_floor)
    }

    fn handle_queue(&mut self){
        if self.queue_is_not_empty(){
            if self.going_up(){self.move_up(); return;}
            if self.going_down(){self.move_down();return;}
            if self.passenger_off_here(){self.let_passengers_off(); return;}
        }

        if self.queue_is_empty(){self.normal_stop(); return;}
    }

    pub fn go_next_floor(&mut self) {
        if self.shall_not_move(){self.stop_motor(); return;}
        if self.allowed_to_move(){self.handle_queue(); return;}
    }

    fn print_status(&self){
        println!("status:{}", self.status.as_str());
    }
    
    //MIDLERTIDIG FUNKSJON
    pub fn door_open_sequence(&mut self) {
        self.set_status(Status::DoorOpen);
        self.door_light(true);

        let handle = thread::spawn(|| {
            // thread::sleep(Duration::from_secs(2)); // Sleep for 2 seconds
            thread::sleep(Duration::from_secs(3)); // Sleep for 2 seconds
            println!("Thread woke up!");
        });
        handle.join().unwrap(); // Wait for the thread to finish

        self.door_light(false);
        self.set_status(Status::Moving);
        self.go_next_floor();
    }
}



impl fmt::Display for Elevator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let addr = self.socket.lock().unwrap().peer_addr().unwrap();
        write!(f, "Elevator@{}({})", addr, self.num_floors)
    }
}

use std::thread::*;
use std::time::*;
use elevio::elev;
use crossbeam_channel as cbc;

use driver_rust::elevio;
use driver_rust::elevio::elev as e;
use elev::Status;

#[cfg(test)] // https://doc.rust-lang.org/book/ch11-03-test-organization.html Run tests with "cargo test"
mod tests {
    use super::*;

    #[test]
    //Function for testing the set status mod
    fn test_set_status() {
        let elev_num_floors = 4;
        let mut elevator = e::Elevator::init("localhost:15657", elev_num_floors).expect("Failed to initialize elevator");
        
        println!("Elevator started:\n{:#?}", elevator);
        
        let poll_period = Duration::from_millis(25);

        let (call_button_tx, call_button_rx) = cbc::unbounded::<elevio::poll::CallButton>();
        {
            let elevator = elevator.clone();
            spawn(move || elevio::poll::call_buttons(elevator, call_button_tx, poll_period));
        }

        let (floor_sensor_tx, floor_sensor_rx) = cbc::unbounded::<u8>();
        {
            let elevator = elevator.clone();
            spawn(move || elevio::poll::floor_sensor(elevator, floor_sensor_tx, poll_period));
        }

        let (stop_button_tx, stop_button_rx) = cbc::unbounded::<bool>();
        {
            let elevator = elevator.clone();
            spawn(move || elevio::poll::stop_button(elevator, stop_button_tx, poll_period));
        }

        let (obstruction_tx, obstruction_rx) = cbc::unbounded::<bool>();
        {
            let elevator = elevator.clone();
            spawn(move || elevio::poll::obstruction(elevator, obstruction_tx, poll_period));
        }

        let mut dirn = e::DIRN_DOWN;


        //IDLE TEST
        elevator.set_status(Status::Idle);
        assert_eq!(elevator.status, Status::Idle);
        println!("Test: IDLE OK");

        //MOVING TEST
        elevator.set_status(Status::Moving);
        assert_eq!(elevator.status, Status::Moving);
        
        elevator.set_status(Status::Idle);
        assert_eq!(elevator.status, Status::Idle);
        println!("Test: MOVING OK");


        //DOOROPEN TEST
        elevator.set_status(Status::DoorOpen);
        assert_eq!(elevator.status, Status::DoorOpen);

        elevator.set_status(Status::DoorOpen);
        assert_eq!(elevator.status, Status::Idle);

        println!("Test: DOOROPEN OK");
        

        //STOP TEST
        elevator.set_status(Status::Stop);
        assert_eq!(elevator.status, Status::Stop);

        elevator.set_status(Status::Idle);
        assert_eq!(elevator.status, Status::Stop);

        elevator.set_status(Status::Moving);
        assert_eq!(elevator.status, Status::Stop);

        elevator.set_status(Status::Stop);
        assert_eq!(elevator.status, Status::Idle);

        println!("Test: STOP OK");

        }

    #[test]
    fn test_go_to_floor() {
        elevator.add_to_queue(3);
        elevator.go_next_floor();
        let elev_num_floors = 4;
        let mut elevator = e::Elevator::init("localhost:15657", elev_num_floors).expect("Failed to initialize elevator");
        
        println!("Elevator started:\n{:#?}", elevator);
        
        let poll_period = Duration::from_millis(25);

        let (call_button_tx, call_button_rx) = cbc::unbounded::<elevio::poll::CallButton>();
        {
            let elevator = elevator.clone();
            spawn(move || elevio::poll::call_buttons(elevator, call_button_tx, poll_period));
        }

        let (floor_sensor_tx, floor_sensor_rx) = cbc::unbounded::<u8>();
        {
            let elevator = elevator.clone();
            spawn(move || elevio::poll::floor_sensor(elevator, floor_sensor_tx, poll_period));
        }

        let (stop_button_tx, stop_button_rx) = cbc::unbounded::<bool>();
        {
            let elevator = elevator.clone();
            spawn(move || elevio::poll::stop_button(elevator, stop_button_tx, poll_period));
        }

        let (obstruction_tx, obstruction_rx) = cbc::unbounded::<bool>();
        {
            let elevator = elevator.clone();
            spawn(move || elevio::poll::obstruction(elevator, obstruction_tx, poll_period));
        }

        let mut dirn = e::DIRN_DOWN;


        let seconds = Duration::from_secs(5);
        let start = SystemTime::now();
                    
        

        loop {
            cbc::select! {
                //tror denne kan bli        
                std::thread::sleep(Duration::new(5, 0));        
                recv(floor_sensor_rx) -> a => {
                    let floor = a.unwrap();
                    elevator.current_floor = floor;
                    println!("Floor: {:#?}", floor);
                    elevator.go_next_floor();  
                },

                match start.elapsed() {
                    Ok(elapsed) if elapsed > seconds => {
                        break;
                    }
                    _ => {},
                }
            }            
        }
        assert_eq!(elevator.current_floor, 3);

        elevator.add_to_queue(1);
        elevator.go_next_floor();

        let seconds = Duration::from_secs(5);
        let start = SystemTime::now();

        loop {
            cbc::select! {
                //tror denne kan bli        
                std::thread::sleep(Duration::new(5, 0));        
                recv(floor_sensor_rx) -> a => {
                    let floor = a.unwrap();
                    elevator.current_floor = floor;
                    println!("Floor: {:#?}", floor);
                    elevator.go_next_floor();  
                },

                match start.elapsed() {
                    Ok(elapsed) if elapsed > seconds => {
                        break;
                    }
                    _ => {},
                }

            }
            
        }
        
        assert_eq!(elevator.current_floor, 1);
        println!("Test: GO TO FLOOR OK");
    }
}
  