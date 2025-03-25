#![allow(dead_code)]
#![warn(unused_variables)]

use std::time::{Duration, SystemTime};
use std::thread;
use crossbeam_channel as cbc;

use crate::modules::elevator_object::*;
use alias_lib::{DIRN_DOWN, DIRN_UP, DIRN_STOP};
use elevator_init::Elevator; 

use super::elevator_status_functions::Status;
use super::cab::Cab;

use crate::modules::udp_functions::udp::*;
use  crate::modules::udp_functions::udp::UdpData;

    impl Cab{
        // Set initial status
        pub fn try_close_door( &mut self, door_tx: cbc::Sender<bool>, obstruction_rx: cbc::Receiver<bool>, elevator: Elevator) -> bool {
            elevator.door_light(true);
            let mut cabclone = self.clone();

            self.set_status(Status::DoorOpen, elevator.clone());
            thread::spawn(move || {
                println!("Doors opened");
                let mut start_time = SystemTime::now();
                loop {
                    // Wait 1 second before attempting to close
                    
                    match obstruction_rx.try_recv() {
                        
                        Ok(true) => {
                             // obstruction: start loop again
                            println!("Obstruction detected, holding doors..");
                            cabclone.status = Status::Obstruction;
                            start_time = SystemTime::now();
                            continue;
                        }
                        Ok(false) =>{
                            cabclone.status = Status::DoorOpen;
                            continue;
                        }
                        Err(cbc::TryRecvError::Empty) => {
                            // No obstruction or nothing received: close door
                            let now = SystemTime::now();
                            if (now.duration_since(start_time).unwrap() > Duration::from_secs(1)) && (cabclone.status != Status::Obstruction) {
                                println!("No obstruction, closing doors");
                                break;
                            }
                            continue;
                        }
                        Err(e) => {
                            println!("Error receiving obstruction: {:?}", e);
                            break;
                        }
                    }
                }
        
                door_tx.send(true).unwrap();
                println!("Doors closed");
            });
            true
        }
    
         
     
    pub fn go_next_floor(&mut self, door_tx: cbc::Sender<bool>, obstruction_rx: cbc::Receiver<bool>, elevator:Elevator) {
        if ((self.status == Status::Moving)||(self.status == Status::Idle))&&(!self.queue.is_empty()){
            if let Some(next_floor) = self.queue.first().map(|first_item| first_item.floor) {
                if next_floor > self.current_floor {
                    self.set_status(Status::Moving, elevator.clone()); 
                    elevator.motor_direction(DIRN_UP);
                    
                } else if next_floor < self.current_floor {
                    self.set_status(Status::Moving, elevator.clone());  //Bytt ut med send status
                    elevator.motor_direction(DIRN_DOWN);
                     
                } else if next_floor == self.current_floor{
                    elevator.motor_direction(DIRN_STOP);  
                    let msg = make_udp_msg(self.id, MessageType::OrderComplete, UdpData::Cab(self.clone()));
                    udp_broadcast(&msg);
                    self.try_close_door(door_tx, obstruction_rx.clone(), elevator.clone());
                }
 
             } else {
                 //self.set_status(Status::Idle);
                 elevator.motor_direction(DIRN_STOP);
             }
         } else {
             elevator.motor_direction(DIRN_STOP);
         }
    }
}
