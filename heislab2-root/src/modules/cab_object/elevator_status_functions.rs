#![allow(dead_code)]
#![warn(unused_variables)]
#[allow(unused_imports)]

use std::fmt;
use crate::modules::elevator_object::*;
use alias_lib::{DIRN_DOWN,DIRN_UP,DIRN_STOP};
use elevator_init::Elevator;
use super::cab::Cab;
use serde::{Deserialize, Serialize};


#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Status{
    Idle,
    Moving,
    DoorOpen,
    Error,
    Stop,
    Obstruction,
}


impl Status{
    pub fn as_str(&self) -> &str{
        match self{
            Status::Idle => "Idle",
            Status::Moving => "Moving",
            Status::DoorOpen => "DoorOpen",
            Status::Error => "Error",
            Status::Stop => "Stop",
            Status::Obstruction => "Obstruction"
        }
        
    }
}

impl Cab{
    pub fn print_status(&self){
        println!("______________________STATUS : {}", self.status.as_str()); //This line got angry if i shortened the rest
    }
    pub fn set_status(&mut self, status: Status, elevator: Elevator){

        match status{
            // Floors are read as u8 0 is hall up, 1 hall down, 2 cab
            Status::Moving => {
                //HVIS DET ER EN ERROR MÅ VI SE OM DET VAR FORRIGE STATUS DA SKAL VI IKKE GJØRE NOE
                match self.status{
                
                    Status::Moving | Status::Idle => {
                        self.status = Status::Moving;
                        let first_item_in_queue = self.queue.first().unwrap();
                        if first_item_in_queue.floor < self.current_floor {
                            self.direction = DIRN_DOWN;
                            
                        } else if first_item_in_queue.floor > self.current_floor{
                            self.direction = DIRN_UP;
                        }
                    }

                    Status::Stop =>{
                        self.status = Status::Stop;
                    }

                    Status::DoorOpen=>{
                        self.status = Status::DoorOpen;
                    }
                    _ =>{
                        //Do Something? 
                    }
                }
            }

            Status::DoorOpen=> {
                match self.status{
                    Status::DoorOpen => {
                        self.status = Status::Idle;
                    }
                    _ => { 
                        self.status = Status::DoorOpen;
                    }
                }
                
            }
            Status::Obstruction=>{
                self.status = Status::Obstruction;
            }

            Status::Idle => {
                match self.status{
                    Status::Stop =>{
                        self.status = Status::Stop;
                        //Do Something? 
                    }
                    _ => {
                        self.status = Status::Idle;
                        //SIER DEN IKKE BEVEGER SEG LENGER
                        self.direction = 0;
                    }
                }
            }

            //From stop you can only swap out by calling stop again
            Status::Stop => {
                match self.status{
                    Status::Stop => {
                        self.status = Status::Idle;

                    }
                    _ => {
                        // KILL ELEVATOR !?
                        self.turn_off_lights(elevator.clone());
                        elevator.motor_direction(DIRN_STOP);
                        self.status = Status::Stop;
                        self.queue.clear();
                        self.print_status();
                    }
                } 
            }

            Status::Error => {
                match self.status{
                    Status::Error =>{
                        self.status= Status::Idle;
                     
                    }
                    _ =>{
                        // KILL ELEVATOR !
                        elevator.motor_direction(DIRN_STOP);
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
        }
        
    }

    pub fn merge_with(&mut self, other: &Cab) {
        // Keep inn_address, out_address, num_floors, and id (these don’t change)
        self.current_floor = other.current_floor;
        self.status = other.status.clone();
        self.direction = other.direction;
        //self.role = other.role.clone();
    }
    
}