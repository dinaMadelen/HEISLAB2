#![allow(dead_code)]
#![warn(unused_variables)]
#[allow(unused_imports)]


#[cfg(test)] // https://doc.rust-lang.org/book/ch11-03-test-organization.html Run tests with "cargo test"
mod tests {
    //use super::poll;

    use std::thread::*;
    use std::time::*;
    use crossbeam_channel as cbc;

    use super::Elevator;
    use super::Status;
    use super::Order;

    use crate::modules::elevator_object::poll;
    

    pub const HALL_UP: u8 = 0;
    pub const HALL_DOWN: u8 = 1;
    pub const CAB: u8 = 2;
    
    pub const DIRN_DOWN: u8 = u8::MAX;
    pub const DIRN_STOP: u8 = 0;
    pub const DIRN_UP: u8 = 1;


    #[test]
    //Function for testing the set status mod
    fn test_set_status() {
        let elev_num_floors = 4;
        let mut elevator = Elevator::init("localhost:15657", elev_num_floors).expect("Failed to initialize elevator");
        
        println!("Elevator started:\n{:#?}", elevator);
        
        let poll_period = Duration::from_millis(25);

        let (call_button_tx, call_button_rx) = cbc::unbounded::<poll::CallButton>();
        {
            let elevator = elevator.clone();
            spawn(move || poll::call_buttons(elevator, call_button_tx, poll_period));
        }

        let (floor_sensor_tx, floor_sensor_rx) = cbc::unbounded::<u8>();
        {
            let elevator = elevator.clone();
            spawn(move || poll::floor_sensor(elevator, floor_sensor_tx, poll_period));
        }

        let (stop_button_tx, stop_button_rx) = cbc::unbounded::<bool>();
        {
            let elevator = elevator.clone();
            spawn(move || poll::stop_button(elevator, stop_button_tx, poll_period));
        }

        let (obstruction_tx, obstruction_rx) = cbc::unbounded::<bool>();
        {
            let elevator = elevator.clone();
            spawn(move || poll::obstruction(elevator, obstruction_tx, poll_period));
        }

        let mut dirn = DIRN_DOWN;


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
        
        let elev_num_floors = 4;
        let mut elevator = Elevator::init("localhost:15657", elev_num_floors).expect("Failed to initialize elevator");
        
        println!("Elevator started:\n{:#?}", elevator);
        
        let poll_period = Duration::from_millis(25);

        let (call_button_tx, call_button_rx) = cbc::unbounded::<poll::CallButton>();
        {
            let elevator = elevator.clone();
            spawn(move || poll::call_buttons(elevator, call_button_tx, poll_period));
        }

        let (floor_sensor_tx, floor_sensor_rx) = cbc::unbounded::<u8>();
        {
            let elevator = elevator.clone();
            spawn(move || poll::floor_sensor(elevator, floor_sensor_tx, poll_period));
        }

        let (stop_button_tx, stop_button_rx) = cbc::unbounded::<bool>();
        {
            let elevator = elevator.clone();
            spawn(move || poll::stop_button(elevator, stop_button_tx, poll_period));
        }

        let (obstruction_tx, obstruction_rx) = cbc::unbounded::<bool>();
        {
            let elevator = elevator.clone();
            spawn(move || poll::obstruction(elevator, obstruction_tx, poll_period));
        }

        let mut dirn = DIRN_DOWN;


        let seconds = Duration::from_secs(5);
        let start = SystemTime::now();
        elevator.add_to_queue(3);
        elevator.go_next_floor();            
        

        loop {
            std::thread::sleep(Duration::new(5, 0));  
            cbc::select! {
                //tror denne kan bli          
                recv(floor_sensor_rx) -> a => {
                    let floor = a.unwrap();
                    elevator.current_floor = floor;
                    println!("Floor: {:#?}", floor);
                    elevator.go_next_floor();  
                }
            }   

            match start.elapsed() {
                Ok(elapsed) if elapsed > seconds => {
                    break;
                }
                _ => {},
            }      
        }
        assert_eq!(elevator.current_floor, 3);

        elevator.add_to_queue(1);
        elevator.go_next_floor();

        let seconds = Duration::from_secs(5);
        let start = SystemTime::now();

        loop {
            std::thread::sleep(Duration::new(5, 0)); 
            cbc::select! {
                //tror denne kan bli        
                       
                recv(floor_sensor_rx) -> a => {
                    let floor = a.unwrap();
                    elevator.current_floor = floor;
                    println!("Floor: {:#?}", floor);
                    elevator.go_next_floor();  
                }

            }
            match start.elapsed() {
                Ok(elapsed) if elapsed > seconds => {
                    break;
                }
                _ => {},
            }
            
        }
        
        assert_eq!(elevator.current_floor, 1);
        println!("Test: GO TO FLOOR OK");
    }
}
  