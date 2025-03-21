#![allow(dead_code)]
#![warn(unused_variables)]

use std::sync::*;
use std::time::Duration;
use std::thread;
use crossbeam_channel as cbc;

use crate::modules::elevator_object::*;
use alias_lib::{DIRN_DOWN, DIRN_UP, DIRN_STOP};
use elevator_init::Elevator; 

use super::elevator_status_functions::Status;
use super::cab::Cab;


    impl Cab{
        // Set initial status
        pub fn try_close_door(&mut self, door_tx: cbc::Sender<bool>, obstruction_rx: cbc::Receiver<bool>, elevator:Elevator) -> bool{
       

        elevator.door_light(true);
        self.set_status(Status::DoorOpen, elevator.clone());
        thread::spawn(move || {
            {
                println!("🚪 Doors opened.");
 
                thread::sleep(Duration::from_secs(1));
                 /*loop{
                     cbc::select!{
                         recv(obstruction_rx)-> a=> {
                             let obstruction = a.unwrap();
                             if obstruction{
                                 thread::sleep(Duration::from_secs(1));
                             } else{
                                 break;
                             }
                         }
                     }
                 }*/

                println!("🚪 Doors closed.");
 
                door_tx.send(true).unwrap();
            } 
            return true;
        });

        elevator.door_light(false);
        return false;

    }
         
     
    pub fn go_next_floor(&mut self, door_tx: cbc::Sender<bool>, obstruction_rx: cbc::Receiver<bool>, elevator:Elevator) {


        if (self.status == Status::Moving) | (self.status == Status::Idle){
            if let Some(next_floor) = self.queue.first().map(|first_item| first_item.floor) {
                if next_floor > self.current_floor {
                    self.set_status(Status::Moving, elevator.clone()); 
                    elevator.motor_direction(DIRN_UP);
                    
                } else if next_floor < self.current_floor {
                    self.set_status(Status::Moving, elevator.clone());  //Bytt ut med send status
                    elevator.motor_direction(DIRN_DOWN);
                     
                } else if next_floor == self.current_floor{
                    elevator.motor_direction(DIRN_STOP);
                    self.turn_off_last_order_light(elevator.clone());  
                    self.queue.remove(0); 
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
