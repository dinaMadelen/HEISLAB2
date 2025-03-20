//--------------------
// Module description
//--------------------
//! This module contains the infitine loop inside main, and everything related 


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
use crate::modules::io::io_loop;

use std::sync::*;


