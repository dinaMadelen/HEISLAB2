//--------------------
// Module description
//--------------------
//! This module contains the objects related to elevator input/output. 
//! The input and output consists of the buttons, floor sensor, and obstruction


//---------
// Imports
//---------
// standard library crates
use std::thread::*;
use std::time::*;

// public crates
use crossbeam_channel as cbc;

// project crates
use crate::modules::elevator_object::poll;
use crate::modules::elevator_object::elevator_init::Elevator;
use crate::modules::elevator_object::io_loop;
