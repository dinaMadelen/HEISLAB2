#![allow(dead_code)]
#![warn(unused_variables)]

use std::fmt;
use std::sync::*;
use std::time::Duration;
use std::thread;
use crossbeam_channel as cbc;

use super::alias_lib::{DIRN_DOWN, DIRN_UP, DIRN_STOP};


use super::elevator_init::Elevator; 
use super::elevator_status_functions::Status;


    impl Elevator{
        // Set initial status
        pub fn try_close_door(&mut self, door_tx: cbc::Sender<bool>, obstruction_rx: cbc::Receiver<bool>) -> bool{
        let status_clone = Arc::clone(&self.status); // ✅ Clone Arc so both threads can use it
        self.door_light(true);
        thread::spawn(move || {
            {
                {
                    let mut status = status_clone.lock().unwrap();
                    *status = Status::DoorOpen;
                 }
                 println!("🚪 Doors opened.");
 
                 thread::sleep(Duration::from_secs(1)); // Simulate door open time
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
                 
                 {
                     let mut status = status_clone.lock().unwrap();
                     *status = Status::Idle;
                 }
                 println!("🚪 Doors closed.");
 
                 door_tx.send(true).unwrap();
             } 
             return true;
         });
         self.door_light(false);
         return false;
     }
         
     
    pub fn go_next_floor(&mut self, door_tx: cbc::Sender<bool>, obstruction_rx: cbc::Receiver<bool>) {
        let true_status= self.status.lock().unwrap();
        let clone_true_status = true_status.clone();
        drop(true_status);
 
        if (clone_true_status == Status::Moving) | (clone_true_status == Status::Idle){
            if let Some(next_floor) = self.queue.first().map(|first_item| first_item.floor) {
                if next_floor > self.current_floor {
                    self.set_status(Status::Moving);
                    self.motor_direction(DIRN_UP);
                     //self.current_floor += 1;
                     
                } else if next_floor < self.current_floor {
                    self.set_status(Status::Moving);
                    self.motor_direction(DIRN_DOWN);
                    //self.current_floor -= 1;
                     
                } else if next_floor == self.current_floor{
                    self.motor_direction(DIRN_STOP);
                    self.turn_off_last_order_light();
                    self.queue.remove(0);
                    self.try_close_door(door_tx, obstruction_rx.clone());
                }
 
             } else {
                 //self.set_status(Status::Idle);
                 self.motor_direction(DIRN_STOP);
             }
         } else {
             self.motor_direction(DIRN_STOP);
         }
    }
}
