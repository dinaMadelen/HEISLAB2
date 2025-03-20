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
       pub fn door_open_sequence(elevator: Arc<Mutex<Self>>) {
        {
            let mut elevator_locked = elevator.lock().unwrap();
            elevator_locked.set_status(Status::DoorOpen);
        } // Unlock the mutex immediately

        let elevator_clone = Arc::clone(&elevator);
        thread::spawn(move || {
            thread::sleep(Duration::from_secs(2)); // Simulate door open time

            let mut elevator_locked = elevator_clone.lock().unwrap();
            elevator_locked.set_status(Status::DoorOpen);
            println!("Thread woke up! Doors closed.");
            elevator_locked.go_next_floor();
        });

        // Main thread continues execution without waiting
        }
        
    



    pub fn go_next_floor(&mut self) {
        if (self.status == Status::Moving) | (self.status == Status::Idle){
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
                    self.set_status(Status::Idle);
                    self.motor_direction(DIRN_STOP);
                    self.turn_off_last_order_light();
                    self.queue.remove(0);
                    self.door_open_sequence(Arc::clone(&self));

            
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