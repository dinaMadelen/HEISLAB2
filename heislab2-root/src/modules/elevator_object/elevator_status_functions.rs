#![allow(dead_code)]
#![warn(unused_variables)]
#[allow(unused_imports)]

use std::fmt;

use crate::modules::elevator_object::*;
use super::alias_lib::{HALL_DOWN, HALL_UP,CAB, DIRN_DOWN, DIRN_UP, DIRN_STOP};
use super::elevator_init::Elevator; 



#[derive(Clone, Debug, PartialEq)]
pub enum Status{
    Idle,
    Moving,
    DoorOpen,
    Error,
    Stop
}

impl Status{
    pub fn as_str(&self) -> &str{
        match self{
            Status::Idle => "Idle",
            Status::Moving => "Moving",
            Status::DoorOpen => "DoorOpen",
            Status::Error => "Error",
            Status::Stop => "Stop"
        }
        
    }
}

impl Elevator{
    pub fn print_status(&self){
        println!("status:{}", self.status.as_str());
    }
    
    pub fn set_status(&mut self, status: Status){
        match status{
            // Floors are read as u8 0 is hall up, 1 hall down, 2 cab
            Status::Moving => {
                //HVIS DET ER EN ERROR MÅ VI SE OM DET VAR FORRIGE STATUS DA SKAL VI IKKE GJØRE NOE
                match self.status{
                
                    Status::Moving | Status::Idle => {
                        self.status = Status::Moving;
                        let first_item_in_queue = self.queue.first().unwrap();
                        if first_item_in_queue.floor < self.current_floor {
                            self.direction = -1;
                            
                        } else if first_item_in_queue.floor > self.current_floor{
                            self.direction = 1;
                        }
                    }

                    Status::Stop =>{
                        self.status = Status::Stop;
                        
                    }
                    _ =>{
                        //Do Something? 
                    }

                }
                //IMPLEMENT LIGHT FUNCTIONALITY HERE

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

            Status::Idle => {
                match self.status{
                    Status::Stop =>{
                        self.status = Status::Stop;
                        //Do Something? 
                    }
                    _ => {
                        self.status = Status::Idle;

                        //SKRUR AV LYSET FOR DER DEN ER
                        if self.direction == -1{
                            self.call_button_light(self.current_floor, HALL_UP , false);
                        }else{
                            self.call_button_light(self.current_floor, HALL_DOWN , false);
                        };
                        self.call_button_light(self.current_floor, CAB , false);

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
                        for f in 0..(self.num_floors) {
                            for c in 0..3 {
                                self.call_button_light(f, c, false);
                            }
                        }

                        self.motor_direction(DIRN_STOP);
                        self.status = Status::Stop;
                        self.queue.clear();
                        self.print_status();
                    }
                } 
            }

            Status::Error => {
                match self.status{
                    Status::Error =>{
                        self.status = Status::Idle;
                    }
                    _ =>{

                        // KILL ELEVATOR !

                        self.motor_direction(DIRN_STOP);
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
}