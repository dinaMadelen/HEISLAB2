
use std::collections::HashMap;
use std::hash::Hash;
use std::net::Ipv4Addr;

use crossbeam_channel::{Receiver, Sender};
use crossbeam_channel as cbc;
use driver_rust::elevio::elev;
use std::time::{Duration, SystemTime};

use crate::memory::{self as mem, Call};
use crate::elevator_interface as elevint;

// Basics of our cyclic counter:
// - It only goes one way, from Nothing to new to confirmed to pendingremoval and then back around
// - To go from nothing to new or from confirmed to pendingremoval only one elevator needs to be in the previous state, ie. we do not need the others to agree
// - To go from new to confirmed or from pendingremoval to nothing we need all the elevators to agree

// There also needs to be some way of dealing with elevators reconnecting with different states, but this is not implemented yet

// Iterates the cyclic counter correctly
fn cyclic_counter(state_to_change: HashMap<Call, mem::CallState>, state_list: &HashMap<Ipv4Addr, mem::State>) -> HashMap<Call, mem::CallState> {
    for mut call in &state_to_change {
        match call.1 {
            mem::CallState::Nothing => {
                // If one of the others has a new order that passed sanity check, change our state to new
                for state in state_list {
                    if *state.1.call_list.get(call.0).unwrap() == mem::CallState::New {
                        call.1 = &mem::CallState::New;
                        break;
                    }
                }
            }
            mem::CallState::New => {
                // If all the others are either new or confirmed, change our state to confirmed
                let mut new = 0;
                let mut confirmed = 0;
                let mut total = 0;
                for state in state_list {
                    total += 1;
                    if *state.1.call_list.get(call.0).unwrap() == mem::CallState::New {
                        new += 1;
                    }
                    else if *state.1.call_list.get(call.0).unwrap() == mem::CallState::Confirmed {
                        confirmed += 1;
                    }
                }
                if (new + confirmed) == total {
                    call.1 = &mem::CallState::Confirmed;
                }
            }
            mem::CallState::Confirmed => {
                // If one of the others has removed an order that passed sanity check, change our state to new
                for state in state_list {
                    if *state.1.call_list.get(call.0).unwrap() == mem::CallState::PendingRemoval {
                        call.1 = &mem::CallState::PendingRemoval;
                        break;
                    }
                }
            }
            mem::CallState::PendingRemoval => {
                // If all the others are either pending or nothing, change our state to nothing
                // it an PendingRemoval is in memory it has to have passed the sanity check
                // TODO check if the sanity check allows other elevators to acsept PendingRemoval of other elevators
                let mut pending = 0;
                let mut nothing = 0;
                let mut total = 0;
                for state in state_list {
                    total += 1;
                    if *state.1.call_list.get(call.0).unwrap() == mem::CallState::PendingRemoval {
                        pending += 1;
                    }
                    else if *state.1.call_list.get(call.0).unwrap() == mem::CallState::Nothing {
                        nothing += 1;
                    }
                }
                if (pending + nothing) == total {
                    call.1 = &mem::CallState::Nothing;
                }
            }
        }
    }
    return state_to_change.clone();
}

// Gets the difference between two call lists
fn difference(old_calls: HashMap<mem::Call, mem::CallState>, new_calls: HashMap<Call, mem::CallState>) -> HashMap<Call, mem::CallState> {
    let mut difference = old_calls.clone();
    for call in old_calls.clone() {
        if new_calls.get(&call.0) == old_calls.get(&call.0) {
            difference.insert(call.0, *new_calls.get(&call.0).unwrap());
        }
    }
    return difference;
}

// Checks whether the changes follow the rules for the cyclic counter
fn filter_changes(differences: HashMap<mem::Call, mem::CallState>, received_state: mem::State, state_list_with_changes: HashMap<Ipv4Addr, mem::State>) -> HashMap<mem::Call, mem::CallState> {
    let mut new_differences = differences.clone();
    for change in differences {
        match change.1 {
            mem::CallState::Nothing => {
                // If the others don't agree, then we cannot update the order to none

                let mut pending = 0;
                let mut new = 0;
                let mut total = 0;
                for state in state_list_with_changes.clone(){
                    total += 1;
                    if *state.1.call_list.get(&change.0).unwrap() == mem::CallState::PendingRemoval {
                        pending += 1;
                    }
                    else if *state.1.call_list.get(&change.0).unwrap() == mem::CallState::New {
                        new += 1;
                    }
                }
                if (pending + new) != total {
                    new_differences.remove(&change.0);
                }
            }
            mem::CallState::New => {
                // Do nothing, new button presses are always legit
            }
            mem::CallState::Confirmed => {
                // If the others don't agree, then we cannot update the order to confirmed

                let mut new = 0;
                let mut confirmed = 0;
                let mut total = 0;
                for state in state_list_with_changes.clone(){
                    total += 1;
                    if *state.1.call_list.get(&change.0).unwrap() == mem::CallState::New {
                        new += 1;
                    }
                    else if *state.1.call_list.get(&change.0).unwrap() == mem::CallState::Confirmed {
                        confirmed += 1;
                    }
                }
                if (new + confirmed) != total {
                    new_differences.remove(&change.0);
                }
            }
            mem::CallState::PendingRemoval => {

                let mut others_agree = false;
                for state in state_list_with_changes.values() {
                    if *state.call_list.get(&change.0.clone()).unwrap() == change.1 {
                        others_agree = true;
                        break;
                    }
                }

                // If the others don't agree or we aren't on the correct floor, we cannot accept the changes
                if received_state.last_floor != change.0.floor || !others_agree {
                    new_differences.remove(&change.0);
                }
            }
        }
    }

    return new_differences;
        
}

// Does as it says on the tin, handles hall calls. Returns hall calls for other elevator
fn handle_hall_calls(old_memory: mem::Memory, received_state: mem::State, my_state: mem::State, memory_request_tx: Sender<mem::MemoryMessage>, state_list_with_changes: HashMap<Ipv4Addr, mem::State>) -> HashMap<mem::Call, mem::CallState> {
     
     // Dealing with hall calls from other elevator

     // Getting new and old calls
     let old_calls: HashMap<mem::Call, mem::CallState> = old_memory.state_list.get(&received_state.id).unwrap().call_list
     .clone()
     .into_iter()
     .filter(|x| x.0.call_type == mem::CallType::Hall(elevint::Direction::Down) || x.0.call_type == mem::CallType::Hall(elevint::Direction::Up))
     .collect();

     let new_calls: HashMap<mem::Call, mem::CallState> = received_state.call_list
     .clone()
     .into_iter()
     .filter(|x| x.0.call_type == mem::CallType::Hall(elevint::Direction::Down) || x.0.call_type == mem::CallType::Hall(elevint::Direction::Up))
     .collect();

     // Getting the difference between the old and new calls to get what calls have changed since last time
     let mut differences = difference(old_calls.clone(), new_calls.clone());

     // Check whether the changed orders are valid or not
     differences = filter_changes(differences, received_state.clone(), state_list_with_changes.clone());


     // Changing our hall calls based on the changes to the received state

     // Getting the relevant calls from my state
     let my_diff: HashMap<mem::Call, mem::CallState> = my_state.call_list.into_iter().filter(|x| differences.contains_key(&x.0)).collect();

     // Running the state machine on only the changed calls
     let my_diff_changed = cyclic_counter(my_diff.clone(), &state_list_with_changes);

     // Extracting the calls that were actually changed to minimize memory changing and avoid errors
     let changed_calls = difference(my_diff, my_diff_changed);

     // Sending the changes to memory one after the other
     for change in changed_calls {
         memory_request_tx.send(mem::MemoryMessage::UpdateOwnCall(change.0, change.1)).unwrap();
     }

     // Returning the hall call changes for the other elevator so it can be included in a state update later
     return differences;
}

// Does as it says on the tin, handles cab calls. Returns cab calls for other elevator
fn handle_cab_calls(old_memory: mem::Memory, received_memory: mem::Memory, memory_request_tx: Sender<mem::MemoryMessage>) -> HashMap<mem::Call, mem::CallState> {
    
    //  Dealing with the cab calls for the other elevator
    // This means filtering out the changes that make no sense
    
    // Checking for cab calls concerning the other elevator
    let old_cab_calls: HashMap<mem::Call, mem::CallState> = old_memory.state_list.get(&received_memory.my_id).unwrap().call_list
    .clone()
    .into_iter()
    .filter(|x| x.0.call_type == mem::CallType::Cab)
    .collect();
    let new_cab_calls: HashMap<mem::Call, mem::CallState> = received_memory.state_list.get(&received_memory.my_id).unwrap().call_list
    .clone()
    .into_iter()
    .filter(|x| x.0.call_type == mem::CallType::Cab)
    .collect();

    // Getting the difference between the old and new cab calls to get what calls have changed since last time
    let mut others_differences_cab = difference(old_cab_calls.clone(), new_cab_calls.clone());

    // Getting a state list with only cab calls from the other elevator
    let mut others_states_for_comparison: HashMap<Ipv4Addr, mem::State> = HashMap::new();
    others_states_for_comparison.insert(received_memory.my_id, received_memory.state_list.get(&received_memory.my_id).unwrap().clone());
    others_states_for_comparison.insert(0.into(), old_memory.state_list.get(&received_memory.my_id).unwrap().clone());

    // Check whether the changed cab calls are valid or not
    others_differences_cab = filter_changes(others_differences_cab, received_memory.state_list.get(&received_memory.my_id).unwrap().clone(), others_states_for_comparison.clone());


    // Dealing with the cab calls for our elevator
    // This means changing the state of our elevators based on the rules of the cyclic counter

    // Checking for cab calls concerning our elevator
    let my_old_cab_calls: HashMap<mem::Call, mem::CallState> = old_memory.state_list.get(&old_memory.my_id).unwrap().call_list
    .clone()
    .into_iter()
    .filter(|x| x.0.call_type == mem::CallType::Cab)
    .collect();
    let my_new_cab_calls: HashMap<mem::Call, mem::CallState> = received_memory.state_list.get(&old_memory.my_id).unwrap().call_list
    .clone()
    .into_iter()
    .filter(|x| x.0.call_type == mem::CallType::Cab)
    .collect();

    // Getting only the cab calls that have changed to minimize overwriting of memory
    let my_differences_cab = difference(my_old_cab_calls.clone(), my_new_cab_calls.clone());

    let mut my_states_for_comparison: HashMap<Ipv4Addr, mem::State> = HashMap::new();
    my_states_for_comparison.insert(old_memory.my_id, old_memory.state_list.get(&old_memory.my_id).unwrap().clone());
    my_states_for_comparison.insert(0.into(), received_memory.state_list.get(&old_memory.my_id).unwrap().clone());

    let my_differences_cab_changed = cyclic_counter(my_differences_cab, &my_states_for_comparison);

    // Sending the changes to memory one after the other
    for change in my_differences_cab_changed {
        memory_request_tx.send(mem::MemoryMessage::UpdateOwnCall(change.0, change.1)).unwrap();
    }

    // Returning the cab call changes for the other elevator so it can be included in a state update later
    return others_differences_cab;
}

fn timeout_check(last_received: HashMap<Ipv4Addr, SystemTime>, memory_request_tx: Sender<mem::MemoryMessage>) -> () {

    // If we have no response from an elevator for a long time, we should not care about it's opinion anymore
    for elevator in last_received {
        if elevator.1.elapsed().unwrap() > Duration::from_secs(3) {
            memory_request_tx.send(mem::MemoryMessage::DeclareDead(elevator.0)).unwrap_or(println!("Cannot declare elevator dead"));
        }
    }
}

// Sanity check and state machine function. Only does something when a new state is received from another elevator
pub fn sanity_check_incomming_message(memory_request_tx: Sender<mem::MemoryMessage>, memory_recieve_rx: Receiver<mem::Memory>, rx_get: Receiver<mem::Memory>) -> () {
    // Setting up a hashmap to keep track of the last time a message was received from each elevator
    let mut last_received: HashMap<Ipv4Addr, SystemTime> = HashMap::new();
    loop {
        cbc::select! {
            recv(rx_get) -> rx => {
                // Getting old memory and extracting my own state
                memory_request_tx.send(mem::MemoryMessage::Request).unwrap_or(println!("Error in requesting memory"));
                let old_memory = memory_recieve_rx.recv().unwrap();
                let my_state = old_memory.state_list.get(&old_memory.my_id).unwrap().clone();

                // Getting new state from rx, extracting both old and new calls for comparison
                let received_memory = rx.unwrap();
                let received_state = received_memory.state_list.get(&received_memory.my_id).unwrap().clone();

                // Setting last received for this elevator to the current time
                last_received.insert(received_state.id, SystemTime::now());

                if !old_memory.state_list.contains_key(&received_memory.my_id) {

                    // Sending the data for the new elevator to memory
                    memory_request_tx.send(mem::MemoryMessage::UpdateOthersState(received_state)).unwrap_or(println!("Error in updating memory"));
                }
                else if old_memory.state_list.get(&received_memory.my_id).unwrap().timed_out {
                    // Holy shit, incomplete code
                    todo!("Do something when a elevator reconnects");

                    // Here we probably need to merge our and their states somehow, but I'm not sure how to do that yet
                }
                else {

                    // Getting a new state list with the changes added
                    let mut state_list_with_changes: HashMap<Ipv4Addr, mem::State> = old_memory.state_list.clone().into_iter().filter(|x| x.1.timed_out == false).collect();
                    state_list_with_changes.insert(received_state.id, received_state.clone());
                    state_list_with_changes.insert(received_state.id, received_state.clone());

                    // Dealing with the new hall calls
                    let differences_in_hall = handle_hall_calls(old_memory.clone(), received_state.clone(), my_state.clone(), memory_request_tx.clone(), state_list_with_changes.clone());

                    // Dealing with the new cab calls
                    let differences_in_cab = handle_cab_calls(old_memory.clone(), received_memory.clone(), memory_request_tx.clone());

                    
                    // Summing up all accepted changes and commiting to memory
                    let mut received_state_new = received_state.clone();
                    for change in differences_in_hall {
                        received_state_new.call_list.insert(change.0, change.1);
                    }
                    for change in differences_in_cab {
                        received_state_new.call_list.insert(change.0, change.1);
                    }

                    // Sending the new state to memory
                    memory_request_tx.send(mem::MemoryMessage::UpdateOthersState(received_state_new)).unwrap_or(println!("Error in updating memory"));
                }
            }

            // If we don't get a new state within 100 ms
            default(Duration::from_millis(100)) => {
                timeout_check(last_received.clone(), memory_request_tx.clone());

                // Getting old memory and extracting my own call list
                let old_memory = memory_recieve_rx.recv().unwrap();
                let my_call_list = old_memory.state_list.get(&old_memory.my_id).unwrap().clone().call_list;

                // Running the state machine on my own calls
                let new_call_list = cyclic_counter(my_call_list.clone(), &old_memory.state_list);

                // Extracting the calls that were actually changed to minimize memory changing and avoid errors
                let changed_calls = difference(my_call_list, new_call_list);

                // Sending the changes to memory one after the other
                for change in changed_calls {
                    memory_request_tx.send(mem::MemoryMessage::UpdateOwnCall(change.0, change.1)).unwrap();
                }
            }
        }
    }
}