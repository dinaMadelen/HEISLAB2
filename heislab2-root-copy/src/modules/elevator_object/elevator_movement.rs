#![allow(dead_code)]
#![warn(unused_variables)]
#[allow(unused_imports)]

use std::fmt;
//use std::io::*;
//use std::net::TcpStream;
//use std::sync::*;
use std::time::Duration;
use std::thread;
use std::sync::{Arc, Mutex};

use super::alias_lib::{DIRN_DOWN, DIRN_UP, DIRN_STOP};
//use crate::modules::elevator_object::*;


use super::elevator_init::Elevator; 
use super::elevator_status_functions::Status;

impl Elevator{
       // Set initial status
        pub fn try_close_door(&mut self) -> bool{
            let status_clone = Arc::clone(&self.status); // ✅ Clone Arc so both threads can use it

        thread::spawn(move || {
            {
                let mut status = status_clone.lock().unwrap();
                *status = Status::DoorOpen;
                println!("🚪 Doors opened.");

                thread::sleep(Duration::from_secs(1)); // Simulate door open time

                *status = Status::Idle;
                println!("🚪 Doors closed.");
            } // ✅ Unlock Mutex after modifying
            return true;
        });
        
        return false;
    }
        

    pub fn door_open_sequence(&mut self) {
        self.try_close_door();
        // Main thread continues execution without waiting
    }
    
    


    pub fn go_next_floor(&mut self) {
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
                    self.door_open_sequence();
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