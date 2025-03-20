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

use crossbeam_channel::RecvError;
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
pub type CallButtonTx   = cbc::Sender<poll::CallButton>;
pub type CallButtonRx   = cbc::Receiver<poll::CallButton>;
pub type CallButtonMsg  = Result<poll::CallButton, RecvError>;

pub type StopButtonTx   = cbc::Sender<bool>;
pub type StopButtonRx   = cbc::Receiver<bool>;
pub type StopButtonMsg  = Result<bool, RecvError>;

pub type FloorSensorTx  = cbc::Sender<u8>;
pub type FloorSensorRx  = cbc::Receiver<u8>;
pub type FloorSensorMsg = Result<u8, RecvError>;

pub type ObstructionTx  = cbc::Sender<bool>;
pub type ObstructionRx  = cbc::Receiver<bool>;
pub type ObstructionMsg = Result<bool, RecvError>;

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

    /// Infinite loop that pools the io channels and handles messages
    fn io_loop(&self, elevator: &Elevator) -> () {
        // loop infinitely
        loop{
            // wait for a ready rx-channel, fetch message and handle message accordingly
            cbc::select! {
                recv(self.call_rx)        -> msg => {handle_call_rx_msg(msg, elevator)},
                recv(self.stop_rx)        -> msg => {handle_stop_rx_msg(msg, elevator)},
                recv(self.floor_rx)       -> msg => {handle_floor_rx_msg(msg, elevator)},
                recv(self.obstruction_rx) -> msg => {handle_obstruction_rx_msg(msg, elevator)},
            }
        }
    }

    fn handle_call_rx_msg(msg: CallButtonMsg, elevator: &Elevator) -> () {
        
    }
    fn handle_stop_rx_msg(msg: StopButtonMsg, elevator: &Elevator) -> () {}
    fn handle_floor_rx_msg(msg: FloorSensorMsg, elevator: &Elevator) -> () {}
    fn handle_obstruction_rx_msg(msg: ObstructionMsg, elevator: &Elevator) -> () {}


}


