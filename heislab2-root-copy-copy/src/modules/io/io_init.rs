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


//-----------
// Constants
//-----------
pub const POLL_PERIOD: Duration = Duration::from_millis(25);


//-------------
// Custom Types
//-------------
pub type CallButtonTx = cbc::Sender<poll::CallButton>;
pub type CallButtonRx = cbc::Receiver<poll::CallButton>;

pub type StopButtonTx = cbc::Sender<bool>;
pub type StopButtonRx = cbc::Receiver<bool>;

pub type FloorSensorTx = cbc::Sender<u8>;
pub type FloorSensorRx = cbc::Receiver<u8>;

pub type ObstructionTx = cbc::Sender<bool>;
pub type ObstructionRx = cbc::Receiver<bool>;


//------------------
// Helper functions
//------------------
/// Initializes and returns a rx channel based on which poll function is give as a argument 
fn create_rx_channel<T, F>(elevator: &Elevator, poll_fn: F) -> cbc::Receiver<T>
    where
        T        : Send + 'static,
        F        : Fn(Elevator, cbc::Sender<T>, Duration) + Send + 'static,
        Elevator : Clone + 'static, 
{
    // initialize cbc channels
    let (tx_channel, rx_channel) = cbc::unbounded::<T>();
    
    let elevator_clone = elevator.clone();

    // initialize polling thread
    spawn(move || poll_fn(elevator_clone, tx_channel, POLL_PERIOD));

    rx_channel
}


//------------------
// Struct varaibles
//------------------
/// Contains the communication channels for the IO
struct IoChannels {
    call_rx        : CallButtonRx,
    stop_rx        : StopButtonRx,
    floor_rx       : FloorSensorRx,
    obstruction_rx : ObstructionRx,
}


//----------------
// Struct methods
//----------------
impl IoChannels {
    /// Initializes the cbc channels and wraps them in a struct
    fn new(elevator: &Elevator) -> IoChannels {
        let io_channels = IoChannels {
            call_rx         : create_rx_channel(elevator, poll::call_buttons),
            stop_rx         : create_rx_channel(elevator, poll::stop_button),
            floor_rx        : create_rx_channel(elevator, poll::floor_sensor),
            obstruction_rx  : create_rx_channel(elevator, poll::obstruction),
        };

        io_channels
    }
}
