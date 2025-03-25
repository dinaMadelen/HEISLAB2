//--------------------
// Module description
//--------------------
//! This module contains the objects related to elevator input/output. 


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

pub type DoorCh = bool;
pub type DoorTx = cbc::Sender<DoorCh>;
pub type DoorRx = cbc::Receiver<DoorCh>;

pub type OrderUpdateCh = Vec<order_init::Order>;
pub type OrderUpdateTx = cbc::Sender<OrderUpdateCh>;
pub type OrderUpdateRx = cbc::Receiver<OrderUpdateCh>;

pub type LightUpdateCh = Vec<order_init::Order>;
pub type LightUpdateTx = cbc::Sender<LightUpdateCh>;
pub type LightUpdateRx = cbc::Receiver<LightUpdateCh>;


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

fn create_door_channel() -> Channel<DoorCh> {
    Channel::<DoorCh>::new()
}

fn create_order_update_channel() -> Channel<OrderUpdateCh> {
    Channel::<OrderUpdateCh>::new()
}

fn create_light_update_channel() -> Channel<LightUpdateCh> {
    Channel::<LightUpdateCh>::new()
}


//------------------
// Struct varaibles
//------------------
#[derive(Clone)]
struct Channel<T> {
    tx : cbc::Sender<T>,
    rx : cbc::Receiver<T>,
}

/// Contains the communication channels for the IO
#[derive(Clone)]
pub struct IoChannels {
    // Receiever channels
    pub call_rx         : CallButtonRx,
    pub stop_rx         : StopButtonRx,
    pub floor_rx        : FloorSensorRx,
    pub obstruction_rx  : ObstructionRx,
    pub door_rx         : DoorRx,
    pub order_update_rx : OrderUpdateRx,
    pub light_update_rx : LightUpdateRx,
    // Transmitter channels
    pub door_tx         : DoorTx,
    pub order_update_tx : OrderUpdateTx,
    pub light_update_tx : LightUpdateTx,
}


//----------------
// Struct methods
//----------------
impl<T> Channel<T> {
    pub fn new() -> Self {
        let (tx, rx) = cbc::unbounded::<T>();
        Self {tx, rx}
    }
}

impl IoChannels {
    /// Initializes the cbc channels and wraps them in a struct
    pub fn new(elevator: &Elevator) -> IoChannels {
        let door_ch               = create_door_channel(); 
        let order_update_ch = create_order_update_channel();
        let light_update_ch =  create_light_update_channel();

        let io_channels = IoChannels {
            // Create reciever channels
            call_rx         : create_rx_channel(elevator, poll::call_buttons),
            stop_rx         : create_rx_channel(elevator, poll::stop_button),
            floor_rx        : create_rx_channel(elevator, poll::floor_sensor),
            obstruction_rx  : create_rx_channel(elevator, poll::obstruction),
            door_rx         : door_ch.rx,
            order_update_rx : order_update_ch.rx,
            light_update_rx : light_update_ch.rx,
            // Create transmitter channe:
            door_tx         : door_ch.tx,
            order_update_tx : order_update_ch.tx,
            light_update_tx : light_update_ch.tx,
        };

        io_channels
    }
}
