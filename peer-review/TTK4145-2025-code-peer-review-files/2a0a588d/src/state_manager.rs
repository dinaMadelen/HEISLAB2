use crate::state_utils::*;
use crate::order_utils::*;
use crate::elevio;
use elevio::poll::CallButton;
use crossbeam_channel as cbc;
use local_ip_address::local_ip;
use crate::UDP::AllEncompassingDataType;
use std::collections::HashMap;

const FLOORS: usize = 4;

pub fn state_manager_main(
                          call_button_rx:           cbc::Receiver::<CallButton>,
                          order_fulfilled_rx:       cbc::Receiver::<CallButton>,
                          from_fsm_rx:              cbc::Receiver<fsm_state>,
                          downlink_rx:              cbc::Receiver<AllEncompassingDataType>,
                          uplink_tx:                cbc::Sender<AllEncompassingDataType>,
                          from_state_manager_tx:    cbc::Sender<fsm_state>,
    ) {

    let mut orders = init_empty_hall_buttons();
    
    let self_name = local_ip().unwrap().to_string();
    let mut states : HashMap<String, State> = HashMap::new();
    states.insert(self_name.clone(), State::init() );
    
    loop {
        cbc::select! {
            recv(call_button_rx) -> data => {
                let call_button = data.unwrap();
                match call_button.call {
                    elevio::elev::HALL_UP | elevio::elev::HALL_DOWN => {
                        confirm_order_at_call_button(&call_button, orders);
                    },
                    elevio::elev::CAB => {
                        states.get_mut(&self_name).unwrap().cabRequests[call_button.floor as usize] = true;
                    },
                    _ => {}
                }
                let hall_requests = format_2_hall_requests(orders);
                let fsm_state = create_fsm_state(states.get(&self_name).unwrap(), hall_requests);

                from_state_manager_tx.send(fsm_state).unwrap();
            },
            recv(from_fsm_rx) -> data => {
                let fsm_state_respons = data.unwrap();
                states.get_mut(&self_name).unwrap().update(fsm_state_respons);
            },
            recv(order_fulfilled_rx) -> data => {
                let fulfilled = data.unwrap();
                match fulfilled.call {
                    0 => {orders[fulfilled.floor as usize].up = OrderStatus::Delivered;}
                    1 => {orders[fulfilled.floor as usize].down = OrderStatus::Delivered;}
                    2 => {states.get_mut(&self_name).unwrap().cabRequests[fulfilled.floor as usize] = false;}
                    _ => {panic!("Tried to fulfill invalid call.")}
                }
            },
            recv(downlink_rx) -> data => {
                // Update states and orders
                
            }
        }
    }
}



fn format_2_elevator_state(ip: String, behaviour: Behaviour, floor: u8, direction: Direction, cab_requests: [bool; FLOORS]) -> serde_json::Value {
    serde_json::json!({
        ip: {
            "behaviour": behaviour.to_string(),
            "floor": floor,
            "direction" : direction.to_string(),
            "cabRequests" : cab_requests
        }
    })
}


pub fn elevator_is_alone(states: &HashMap::<String, State>) -> bool {
    if states.len() == 1 {
        return true
    }
    false
}
