use std::usize;
use crossbeam_channel as cbc;
use network_rust::udpnet::peers::PeerUpdate;

use crate::elevio::poll::CallButton;
use crate::execution::elevator::{Dirn, ElevatorBehaviour, N_FLOORS};
use crate::interface::HallRequestMatrix;

pub const N_ELEVATORS :usize = 3;

#[path = "./cost.rs"]
mod cost;

#[derive(Clone,Copy, Debug, serde::Deserialize, serde::Serialize)]
pub struct ElevatorArgument {
    pub behaviour: ElevatorBehaviour,
    pub floor: usize,
    pub direction: Dirn,
    #[serde(rename = "cabRequests")]
    pub cab_requests: [bool; N_FLOORS],
}

impl ElevatorArgument{
    pub fn init() -> Self{
        ElevatorArgument{
            behaviour:ElevatorBehaviour::Idle,
            floor:0,
            direction:Dirn::Stop,
            cab_requests:[false;N_FLOORS],
        }
    }
}

pub struct Controller {
    pub elevator_number: u8,
    
    pub hall_requests: HallRequestMatrix,
    pub active_elevators:[bool; N_ELEVATORS],
    pub elevator_states: [ElevatorArgument;N_ELEVATORS],

    pub ctc_reconnecting_hall_requests_tx: cbc::Sender<HallRequestMatrix>,
    pub ctc_inactive_state_tx: cbc::Sender<(u8,bool)>
}

impl  Controller {

    pub fn init(elevator_id: u8, ctc_reconnecting_hall_requests_tx:cbc::Sender<HallRequestMatrix>, ctc_inactive_state_tx: cbc::Sender<(u8, bool)>) -> Self{
        let mut active_list = [false; N_ELEVATORS];
        active_list[elevator_id as usize] = true;
        Controller{
            elevator_number: elevator_id,
            hall_requests:[[false; 2]; N_FLOORS],
            active_elevators:active_list,
            elevator_states:[ElevatorArgument::init(); N_ELEVATORS],

            ctc_reconnecting_hall_requests_tx:ctc_reconnecting_hall_requests_tx,
            ctc_inactive_state_tx:ctc_inactive_state_tx
            }
    }

    /// Determines and returns best hall request distribution between available elevators based on controller states
    pub fn assign_requests(&self) -> cost::HallRequestsAssignments{

        // Prepping hall request overview and ElevatorArguments of active elevators as distribution function-parameters
        let mut input = cost::HallRequestsStates{
            hall_requests:self.hall_requests,
            states: cost::ElevatorArguments::new()
        };

        for i in 0..N_ELEVATORS{
            if self.active_elevators[i]{
                input.states.insert(i,self.elevator_states[i]); //Key matches elevatornumber
            }
        }

        if let Ok(distributed_orders) = cost::run_hall_request_assigner(input){
            return distributed_orders
        }else{
            cost::HallRequestsAssignments::new()
        }
    }
    
    /// Calculate and sends hierarchy position to broadcast-thread
    pub fn calculate_hierarchy_position(peer_update: &PeerUpdate, elevator_number: u8, hierarchy_position_tx: &cbc::Sender<u8>)-> u8 {
        let mut active_elevator_list = [false; N_ELEVATORS];
        active_elevator_list[elevator_number as usize] = true;
        for peer in &peer_update.peers {
            if let Ok(peer_u8) = peer.parse::<usize>() {
                active_elevator_list[peer_u8] = true;
            }
        }
        let mut position = 0;
        for i in 0..(elevator_number as usize) {
            if active_elevator_list[i] == true{
                position += 1;
            }
        }
        hierarchy_position_tx.send( position).expect("hierarchy_position_tx");
        position
    }

    /// Updates the information aboout elevator state held by controller
    pub fn update_elevator_argument(&mut self, elevator_number: u8, elevator_argument: ElevatorArgument){ 
        self.elevator_states[elevator_number as usize] = elevator_argument;
    }
    
    /// Sets order to active/inactive based on input
    pub fn set_hall_order_state(&mut self, call_button: CallButton, state: bool) -> bool{
        let floor = call_button.floor as usize;
        let call = call_button.call as usize;
        if self.hall_requests[floor][call] != state{
            self.hall_requests[floor][call] = state;
            return true;
        }
        false
    }

    /// Sets new peers to active and lost peers as inactive in the controllers list
    pub fn update_active_list(&mut self, peer_update: &PeerUpdate) {
        if let Some(peer) = &peer_update.new {
            if let Ok(peer_index) = peer.parse::<usize>() {
                self.active_elevators[peer_index] = true;
            }
        }
    
        for peer in &peer_update.lost {
            if let Ok(peer_index) = peer.parse::<usize>() {
                self.active_elevators[peer_index] = false;
            }
        }
    }

    /// Set all states that is true in the input matrix to true in the controllers hall_request matrix
    pub fn insert_hall_request_matrix(&mut self, hall_requests: HallRequestMatrix)->bool{
        let mut change  = false;
        for floor in 0..N_FLOORS{
            for btn_type in 0..2{
                if hall_requests[floor][btn_type] != self.hall_requests[floor][btn_type]{
                    self.hall_requests[floor][btn_type] = hall_requests[floor][btn_type];
                    change = true
                } 
            }
        }
        change
    }

    /// Toggels active/inactive in controllers overview, intended for obstruction switch toggle
    pub fn set_local_elevator_active_status(&mut self,active_state: bool){
        self.active_elevators[self.elevator_number as usize] = active_state;
        // Informs other controllers
        self.ctc_inactive_state_tx.send((self.elevator_number, true)).expect("ctc_inactive_state_tx");
    }

    ///Sets active/inactive elevator based on input, intended for input over UDP about obsturction switch for other elevators
    pub fn set_external_elevator_active_status(&mut self, elevator_number: u8, active_state: bool) -> bool{
        let index = elevator_number as usize;
        if self.active_elevators[index] != active_state{
            self.active_elevators[elevator_number as usize] = active_state;
            return true
        }
        false
    }
    
    /// Sets active hall requests to 0, intended for transition to disconnected state
    pub fn wipe_hall_request_orders(&mut self){
        self.hall_requests = [[false; 2]; N_FLOORS];
    }
    /// Sends all active hall requests to other controllers, intended for transition disconnected to normal operation
    pub fn send_hall_request_to_controllers(&mut  self){
        self.ctc_reconnecting_hall_requests_tx.send(self.hall_requests).expect("ctc_reconnecting_hall_requests_tx");
    }
}
