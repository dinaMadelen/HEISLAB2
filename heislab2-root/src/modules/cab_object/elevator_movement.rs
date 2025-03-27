#![allow(dead_code)]
#![warn(unused_variables)]

use std::time::{Duration, SystemTime};
use std::thread;
use std::collections::VecDeque;
use crossbeam_channel as cbc;

use crate::modules::elevator_object::*;
use alias_lib::{CAB,HALL_DOWN,HALL_UP,DIRN_DOWN, DIRN_UP, DIRN_STOP};
use elevator_init::Elevator; 

use super::elevator_status_functions::Status;
use super::cab::Cab;

impl Cab{
    // Set initial status
    pub fn try_close_door( &mut self, door_tx: cbc::Sender<bool>, obstruction_rx: cbc::Receiver<bool>, elevator: Elevator) -> bool {
        elevator.door_light(true);
        let mut cabclone = self.clone();

        self.set_status(Status::DoorOpen, elevator.clone());
        thread::spawn(move || {
            println!("Doors opened");                let mut start_time = SystemTime::now();
            loop {
                // Wait 1 second before attempting to close
                    
                match obstruction_rx.try_recv() {
                        
                    Ok(true) => {
                         // obstruction: start loop again
                        println!("Obstruction detected, holding doors..");
                        cabclone.status = Status::Obstruction;
                        start_time = SystemTime::now();                            continue;
                        }
                    Ok(false) =>{
                        cabclone.status = Status::DoorOpen;
                        continue;
                        }
                    Err(cbc::TryRecvError::Empty) => {
                        // No obstruction or nothing received: close door
                        let now = SystemTime::now();
                        if (now.duration_since(start_time).unwrap() > Duration::from_secs(3)) && (cabclone.status != Status::Obstruction) {
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
         // Only attempt to change floors if we are moving or idle and there is at least one order.
    if (self.status == Status::Moving || self.status == Status::Idle) && !self.queue.is_empty() {
        // If we are idle (and about to depart), update last_served_floor.
        if self.status == Status::Idle {
            // Update last_served_floor here before starting to move.
            self.last_served_floor = self.current_floor;
        }
        
        // The "target" floor for stop decisions:
        // When idle, we use the current floor. When moving, we use the last floor we stopped at.
        let effective_floor = if self.status == Status::Moving {
            self.last_served_floor
        } else {
            self.current_floor
        };

        if let Some(next_floor) = self.queue.first().map(|first_item| first_item.floor) {
            // Check if any order in the queue matches the effective floor.
            let should_stop = self.queue.iter().any(|order| {
                order.floor == effective_floor &&
                (
                    (self.direction == DIRN_UP && order.order_type == HALL_UP) ||
                    (self.direction == DIRN_DOWN && order.order_type == HALL_DOWN) ||
                    (order.order_type == CAB)
                )
            });

            if should_stop {
                // Find the matching order and bring it to the front.
                if let Some((db_order_index, _)) = self.queue.iter().enumerate().find(|(_, order)| {
                    order.floor == effective_floor &&
                    (
                        (self.direction == DIRN_UP && order.order_type == HALL_UP) ||
                        (self.direction == DIRN_DOWN && order.order_type == HALL_DOWN) ||
                        (order.order_type == CAB)
                    )
                }) {
                    let driveby_order = self.queue.remove(db_order_index);
                    self.queue.insert(0, driveby_order);
                    println!(
                        "Stopping at floor {} because order in queue matches effective floor {}.",
                        effective_floor, effective_floor
                    );
                    elevator.motor_direction(DIRN_STOP);
                    self.try_close_door(door_tx, obstruction_rx.clone(), elevator.clone());
                    // Update current_floor now that we've stopped.
                    self.current_floor = effective_floor;
                }
            } else {
                // If no order demands a stop at the effective floor, move toward the next order.
                if next_floor > self.current_floor {
                    self.set_status(Status::Moving, elevator.clone());
                    elevator.motor_direction(DIRN_UP);
                } else if next_floor < self.current_floor {
                    self.set_status(Status::Moving, elevator.clone());
                    elevator.motor_direction(DIRN_DOWN);
                } else if next_floor == self.current_floor {
                    // Should only happen if we have just arrived.
                    elevator.motor_direction(DIRN_STOP);
                    self.try_close_door(door_tx, obstruction_rx.clone(), elevator.clone());
                    self.current_floor = next_floor;
                }
            }
        } else {
            elevator.motor_direction(DIRN_STOP);
        }
    } else {
        // If there are no orders, just ensure the motor is stopped.
        elevator.motor_direction(DIRN_STOP);
    }
}
}    
