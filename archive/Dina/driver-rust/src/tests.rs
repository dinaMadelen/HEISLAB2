use std::thread::*;
use std::time::*;
use elevio::elev;
use crossbeam_channel as cbc;

use driver_rust::elevio;
use driver_rust::elevio::elev as e;
use elev::Status;

#![allow(dead_code)]

#[cfg(test)] // https://doc.rust-lang.org/book/ch11-03-test-organization.html Run tests with "cargo test"
mod tests {
    use super::*;
    let elev_num_floors = 4;
    let mut elevator = e::Elevator::init("localhost:15657", elev_num_floors).expect("Failed to initialize elevator");


    #[test]
    //Function for testing the set status mod
    fn test_set_status() {

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
  