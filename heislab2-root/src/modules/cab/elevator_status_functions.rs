#![allow(dead_code)]
#![warn(unused_variables)]
#[allow(unused_imports)]

use std::fmt;
use super::alias_lib::{HALL_DOWN, HALL_UP,CAB, DIRN_STOP};
use crate::modules::cab::Cab;


#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
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

impl Cab{
    pub fn print_status(&self){
        let true_status= self.status.lock().unwrap();
        let clone_true_status = true_status.clone();
        drop(true_status);

        let cloned_true_status_as_str = clone_true_status.as_str();

        println!("status:{}", cloned_true_status_as_str); //This line got angry if i shortened the rest
    }
    pub fn set_status(&mut self, status: Status){
        let true_status= self.status.lock().unwrap();
        let clone_true_status = true_status.clone();
        drop(true_status);

        match status{
            // Floors are read as u8 0 is hall up, 1 hall down, 2 cab
            Status::Moving => {
                //HVIS DET ER EN ERROR MÅ VI SE OM DET VAR FORRIGE STATUS DA SKAL VI IKKE GJØRE NOE
                match clone_true_status{
                
                    Status::Moving | Status::Idle => {
                        let mut true_status= self.status.lock().unwrap();
                        *true_status = Status::Moving;
                        drop(true_status);

                        let first_item_in_queue = self.queue.first().unwrap();
                        if first_item_in_queue.floor < self.current_floor {
                            self.direction = -1;
                            
                        } else if first_item_in_queue.floor > self.current_floor{
                            self.direction = 1;
                        }
                    }

                    Status::Stop =>{
                        let mut true_status= self.status.lock().unwrap();
                        *true_status = Status::Stop;
                        drop(true_status);
                    }
                    Status::DoorOpen=>{
                        let mut true_status= self.status.lock().unwrap();
                        *true_status = Status::DoorOpen;
                        drop(true_status);
                    }
                    _ =>{
                        //Do Something? 
                    }

                }
                //IMPLEMENT LIGHT FUNCTIONALITY HERE

            }

            Status::DoorOpen=> {
                match clone_true_status{
                    Status::DoorOpen => {
                        let mut true_status= self.status.lock().unwrap();
                        *true_status = Status::Idle;
                        drop(true_status);
                    }
                    _ => {
                        let mut true_status= self.status.lock().unwrap();
                        *true_status = Status::DoorOpen;
                        drop(true_status);
        
                    }
                }
                
            }

            Status::Idle => {
                match clone_true_status{
                    Status::Stop =>{
                        let mut true_status= self.status.lock().unwrap();
                        *true_status = Status::Stop;
                        drop(true_status);
                        //Do Something? 
                    }
                    Status::DoorOpen =>{
                        let mut true_status= self.status.lock().unwrap();
                        *true_status = Status::DoorOpen;
                        drop(true_status);
                    }
                    _ => {
                        let mut true_status= self.status.lock().unwrap();
                        *true_status = Status::Idle;
                        drop(true_status);

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
                match clone_true_status{
                    Status::Stop => {
                        let mut true_status= self.status.lock().unwrap();
                        *true_status = Status::Idle;
                        drop(true_status);
                    }
                    _ => {
                        // KILL ELEVATOR !?
                        self.turn_off_lights();

                        self.motor_direction(DIRN_STOP);
                        let mut true_status= self.status.lock().unwrap();
                        *true_status = Status::Stop;
                        drop(true_status);

                        self.queue.clear();
                        self.print_status();
                    }
                } 
            }

            Status::Error => {
                match clone_true_status{
                    Status::Error =>{
                        let mut true_status= self.status.lock().unwrap();
                        *true_status = Status::Idle;
                        drop(true_status);
                    }
                    _ =>{
                        // KILL ELEVATOR !
                        self.motor_direction(DIRN_STOP);

                        let mut true_status= self.status.lock().unwrap();
                        *true_status = Status::Error;
                        drop(true_status);

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