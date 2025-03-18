#![allow(dead_code)]
#![warn(unused_variables)]

use std::fmt;
use std::io::*;
use std::net::TcpStream;
use std::sync::*;
use std::time::Duration;
use std::thread;

use super::alias_lib::{HALL_DOWN, HALL_UP,CAB, DIRN_DOWN, DIRN_UP, DIRN_STOP};
use crate::modules::elevator_object::*;


use super::elevator_init::Elevator; 
use super::elevator_status_functions::Status;

impl Elevator{
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

    pub fn go_next_floor(&mut self) {
        if ((self.status == Status::Moving) | (self.status == Status::Idle)){
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
}