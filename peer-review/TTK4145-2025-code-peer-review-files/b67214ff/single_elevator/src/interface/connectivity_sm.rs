use std::{process, thread};
use crossbeam_channel as cbc;
use network_rust::udpnet::{self, peers::PeerUpdate};
use crate::logic::controller::Controller;

//This file contains states, triggers and transitions between states for the connectivity, which affects the controller. Functions to keep track
//of the connectivity of other controller-elevator pairs are also included here.

//NOTE: connectivity_sm are only to be used in interface/cotnroller.rs, thus all states etc. are not pub other than to controller-functions.

//States
struct NormalOperation;
struct Disconnected;
struct Inactive;
trait State {
    fn transition(self: Box<Self>, controller: &mut Controller, event: (Event,bool)) -> Box<dyn State>;
}

//Triggers
pub enum Event{
    ObstructionSwitch,
    NoPeers,
    NewPeer
}

//================ Transitions ==========================

impl State for Disconnected {
    fn transition(self: Box<Self>, controller: &mut Controller, event: (Event,bool)) -> Box<dyn State> {
        match event.0 {
            Event::NewPeer =>{
                controller.send_hall_request_to_controllers();
                println!("Normal Operation");

                Box::new(NormalOperation{})
            } 
            _ => self, // Stay in Disconnected
        }
    }
}

impl State for Inactive {
    fn transition(self: Box<Self>, controller: &mut Controller,  event: (Event, bool)) -> Box<dyn State> {
        match event.0 {
            Event::NoPeers => {
                println!("Disconnected");
                Box::new(Disconnected {})}, // Transition to Disconnected
            Event::ObstructionSwitch => {
                if event.1 == false{
                    controller.set_local_elevator_active_status(true); //Note obbstructionfalse = active state true
                    Box::new(NormalOperation{})
                }else{
                    self
                }
            }
            _ => self, // Stay in Inactive
        }
    }
}

impl State for NormalOperation{
    fn transition(self: Box<Self>, controller: &mut Controller,  event: (Event,bool)) -> Box<dyn State> {
        match event.0 {
            Event::NoPeers => {
                //Initiate run single elevator
                controller.wipe_hall_request_orders();
                println!("Disconnected");
                Box::new(Disconnected{})}

            Event::ObstructionSwitch => {
                if event.1 == true {
                    controller.set_local_elevator_active_status(false);
                    Box::new(Inactive{})
                }else{ 
                    self}
                }
            _ => self
        }
    }
}

//============Peer updates================

pub fn send_alive(elevator_number: u8, peer_send_port: u16){
    let peer_port = peer_send_port;//19738;
    //Sender for peer discovery
    let (peer_tx_enable_tx,peer_tx_enable_rx) = cbc::unbounded::<bool>();
    {
        thread::spawn(move || {
            if udpnet::peers::tx(peer_port, elevator_number.to_string(), peer_tx_enable_rx).is_err() {
                // crash program if creating the socket fails (`peers:tx` will always block if the
                // initialization succeeds)
                process::exit(1);
            }
        });
        thread::spawn(move || {
            peer_tx_enable_tx.send(true).expect("peer_tx_enable_tx"); // Can stop sending by setting this to false
            loop {
                
            }
        });
    }

}

pub fn recieve_online_statuses(peer_update_tx: cbc::Sender<PeerUpdate>,peer_listen_port: u16){
    let peer_port = peer_listen_port;//19738;

    {
        thread::spawn(move || {
            if udpnet::peers::rx(peer_port, peer_update_tx).is_err() {
                // crash program if creating the socket fails (`peers:rx` will always block if the
                // initialization succeeds)
                process::exit(1);
            }
        });
    }
}

//=================State machine implementation technicalities================

/// Structure to hold a state of dynamic type (supported by Box and the common trait State)
pub struct Node {
    state: Box<dyn State>,
}

impl Node {
    pub fn new() -> Node {
        Node { state: Box::new(Disconnected {}) } // Start in Inactive
    }
    /// Handles state-machine transitions and returns true on state changes
    pub fn handle_transition(&mut self, controller: &mut Controller, event: (Event, bool)) -> bool {
        let old_state = std::mem::replace(&mut self.state, Box::new(Inactive {})); //Dummy replcement to handle ownership
        let new_state = old_state.transition(controller, event);
    
        // Check if state has changed
        let state_changed = !std::ptr::eq(&*self.state, &*new_state);
    
        // Assign the new state
        self.state = new_state;
    
        state_changed
    }
}
