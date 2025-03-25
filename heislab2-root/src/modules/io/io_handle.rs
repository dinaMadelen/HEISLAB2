//--------------------
// Module description
//--------------------
//! This module contains handler functions for elevator input/output


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
use crate::modules::order_object::order_init;
use crate::modules::io::io_init::*;


//-----------
// Functions
//-----------
// pub fn 