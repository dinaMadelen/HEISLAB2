use crossbeam_channel as cbc;
use std::time;
use std::thread;

use crate::elevio::poll::CallButton;
use crate::logic::controller::{Controller, ElevatorArgument};
use super::HallRequestMatrix;
use network_rust::udpnet;

// Including the state machine elements
#[path = "./connectivity_sm.rs"]
mod connectivity_sm;

/// Sends well distributed orders to elevators
fn distribute_and_send_orders(controller:&Controller, cte_hall_request_matrix_tx: &cbc::Sender<(u8,HallRequestMatrix)>){
    let distributed_orders = controller.assign_requests();
    

    for i in distributed_orders.keys() {
        if let Some(requests) = distributed_orders.get(i) {
            cte_hall_request_matrix_tx.send((*i  as u8, *requests)).expect("cte_hall_request_matrix_tx");
            thread::sleep(time::Duration::from_millis(5)); 
        }
    }
}

/// Runs all controller functionality
pub fn run_controller(
    elevator_id: u8,
    cte_hall_request_matrix_tx: cbc::Sender<(u8,HallRequestMatrix)>, 
    elevator_argument_rx: cbc::Receiver<(u8, ElevatorArgument)>,
    order_state_rx: cbc::Receiver<(CallButton, bool)>, 
    obstruction_rx: cbc::Receiver<bool>,
    hierarchy_position_tx: cbc::Sender<u8>,
    ctc_reconnecting_hall_requests_tx: cbc::Sender<HallRequestMatrix>,
    ctc_inactive_state_tx: cbc::Sender<(u8,bool)>,
    ctc_elevator_argument_tx: cbc::Sender<(u8,ElevatorArgument)>,
    ctc_network_hall_requests_rx: cbc::Receiver<HallRequestMatrix>,
    ctc_network_inactive_state_rx: cbc::Receiver<(u8, bool)>,
    ctc_network_elevator_argument_rx: cbc::Receiver<(u8,ElevatorArgument)>,
    peer_listen_port:u16,
    peer_send_port: u16
){

    let mut controller = Controller::init(elevator_id, ctc_reconnecting_hall_requests_tx, ctc_inactive_state_tx);

    // ====== Setting up Connectivity State-machine ========
    let mut node = connectivity_sm::Node::new();
    connectivity_sm::send_alive(elevator_id,peer_send_port);
    let (peer_update_tx, peer_update_rx) = cbc::unbounded::<udpnet::peers::PeerUpdate>();
    connectivity_sm::recieve_online_statuses(peer_update_tx, peer_listen_port);

    loop {
        cbc::select! {
            // ==== Elevator inputs ======
            recv(order_state_rx) -> a => {
                let order_state = a.unwrap();
                if controller.set_hall_order_state(order_state.0, order_state.1){
                    distribute_and_send_orders(&controller, &cte_hall_request_matrix_tx);
                }
            }
            recv(elevator_argument_rx) -> a => {
                let elevator_argument = a.unwrap();
                controller.update_elevator_argument(elevator_argument.0, elevator_argument.1);
            }

            // ===== Inputs from other controllers ======
            
            recv(ctc_network_hall_requests_rx) -> a => {
                let hall_requests =  a.unwrap();
                if controller.insert_hall_request_matrix(hall_requests){
                   distribute_and_send_orders(&controller, &cte_hall_request_matrix_tx); 
                }

            }
            recv(ctc_network_inactive_state_rx) -> a =>{
                
                let (elevator_number, inactive_state) = a.unwrap();
                if controller.set_external_elevator_active_status(elevator_number, inactive_state){
                    distribute_and_send_orders(&controller, &cte_hall_request_matrix_tx);
                }
            }
            recv(ctc_network_elevator_argument_rx) -> a =>{
                let elevator_argument = a.unwrap();
                controller.update_elevator_argument(elevator_argument.0, elevator_argument.1);
            }


            //========== Connectivity State machine =================

            //Event: Obstruction
            recv(obstruction_rx) -> a  =>{
                let obstruction_bool = a.unwrap();
                if node.handle_transition(&mut controller, (connectivity_sm::Event::ObstructionSwitch,obstruction_bool)){
                    distribute_and_send_orders(&controller, &cte_hall_request_matrix_tx);
                }

            }
            //Event: NoPeers or NewPeer 
            recv(peer_update_rx) -> a => {
                let peer_update = a.unwrap();
                ctc_elevator_argument_tx.send( (controller.elevator_number,controller.elevator_states[controller.elevator_number as usize])).unwrap();
                controller.update_active_list(&peer_update);
                Controller::calculate_hierarchy_position(&peer_update, controller.elevator_number, &hierarchy_position_tx);

                // Find state machine trigger-Event from peerUpdate
                let mut event: Option<(connectivity_sm::Event, bool)> = None;

                if peer_update.peers.is_empty() {
                    hierarchy_position_tx.send(0).expect("hierarchy_position_tx");
                    event = Some((connectivity_sm::Event::NoPeers, true)); // Disconnected
                } else if peer_update.new.is_some() { 
                    event = Some((connectivity_sm::Event::NewPeer, true));
                }

                // Running State Machine
                if let Some(ev) = event {
                    if node.handle_transition(&mut controller, ev) {
                        distribute_and_send_orders(&controller, &cte_hall_request_matrix_tx);
                    }
                }
            }
        }
    }
}