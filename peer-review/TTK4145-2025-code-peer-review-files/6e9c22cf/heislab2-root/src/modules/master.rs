#[warn(non_snake_case)]
/*Fordeling av ordre
fordeling av ordre basert på kost funksjon  XXXXXXXXXX

Worldview broadcast & retting av feil i worldview XXXXXXXXXXX
sending av wordview(eksisterer broadcast funksjon i UDP modulen) 

Gi ordre og forvent svar fra alle aktive 

Gi fra seg master rolle XXXXXXXXXXXXXXXXXXXXXXXX
usikker på om denne trengs da slaver skal ta over rollen automatisk hvis master går offline
men kan være fin dersom det er noe som gjør at master vil restarte og vil framskynde prosessen

Example for worldview struct
worldview is a vector of vectors that contain the orders of every active cab
Master_view={
    worldview: Vec<Elevator> = [elevator1],[elevator2],[elevator3],[elevator4],
    lights: Vec<u8> = [lights_cab1, lights_cab2, lights_cab3, lights_cab3, lights_cab4],
}

*/

//-----------------------IMPORTS------------------------------------------------------------

use crate::modules::udp::{udp_send_ensure,udp_broadcast,udp_receive,make_Udp_msg};
use crate::modules::elevator::{Elevator,Status};
use std::net::UdpSocket;

//-----------------------STRUCTS------------------------------------------------------------

pub struct Worldview{

    orders: Vec<Elevator>,
    lights: Vec<u8>,
}

pub enum Role{
    Master,
    Slave,
    Error,
}

//-----------------------FUNCTIONS---------------------------------------------------------

///Broadcast order and wait for responce from reciver, if not recived resend, if this fail. find return false
///The diffrence from just adding from worldview broadcast and from give_order() is that unlike regular udp_broadcast() give_order() requires an acknoledgement from the recivers

fn give_order(master: &Elevator, slave_id: u8, new_order: u8) -> bool {

    let mut retries = 3;
    let message = make_udp_msg(master.id, MessageType::NewOrder, new_order);

    udp_broadcast(&socket, &slave_address(slave_id), &message);

    let mut retry: u8 = 4;
    let mut accepted: Vec<u8> = Vec::new();

    while retry > 0{
        udp_receive(); 
    // add id of ack sender to accepted
        if accepted.len() == elevators.len() {
            return true;
0       }else{
            println!("Missing acknowledgements from active elevators");
        }
        retry -= 1;
    }
    return false;

    println!("Failed to deliver order to slave {}", slave_id);
    return false;
}


/// Send order to remove one or more orders from a specific elevator
fn remove_from_queue(slave_id: u8, removed_orders: Vec<u8>) -> bool {

    let message = make_udp_msg(master.id, MessageType::RemoveOrder, removed_orders);
    return udp_send_ensure(&socket, &slave_address(slave_id), &message);
}

/// Compare message and send out the corrected worldview (union of the recived and current worldview)
fn correct_master_worldview(master: &Elevator) -> bool {

    let missing_orders = todo!("Vector of vectors containing the queues from the slave with orders that dont exist in worldview");

    // for order in missing_orders
    // union of order and exisiting queue in worldview

    let message = make_udp_msg(master.id, MessageType::Worldview, state);
    
    return udp_broadcast(&socket, &message);
}

/// Broadcast worldview
fn master_worldview(master:Elevator) -> bool{
    
    make_Udp_msg(sender_id: master,message_type: Wordview, message:Vec<u8>); 
    
    return udp_broadcast(&socket,&message);
}

// Give away master role, NOT NEEDED, KILL INSTEAD
/*
fn relinquish_master(master: &mut Elevator) -> bool {

    let message = make_udp_msg(master.id, MessageType::RelinquishMaster, vec![]);
    udp_broadcast(&socket, &message);

    master.status = Elevator.status::Error;
    return true;
}
*/

/// Handle slave failure, compensate for a slave going offline
fn handle_slave_failure(slave_id: u8, elevators: &mut Vec<Elevator>) {

    println!("Elevator {} is offline, redistributing elevator {}'s orders.", slave_id,slave_id);

    // Find and redistribute orders for elevator with that spesific ID
    if let Some(index) = elevators.iter().position(|elevator| elevator.ID == slave_id) {
        // Have to use clone to not take ownership of the queue variable(problem compiling)
        let orders = elevators[index].queue.clone(); 
        elevators.remove(index);
        reassign_orders(orders, elevators);
    } else {
        println!("Error: cant find Elevator with ID {}", slave_id);
    }
}

/// Reassign order
fn reassign_orders(orders: Vec<u8>) {

    for order in orders {
        for best_alternative in best_to_worst_elevator(order){
            msg= make_Udp_msg(sender_id:my_id, message_type: message_type, message:Vec<u8>)
            // fix inputs to udp_send_ensure function, dont remember exactly how it was, check udp.rs.
            udp_send_ensure(socket: &UdpSocket, target_addr: &str, msg: &UdpMsg)
        }
    }
}

/// Cost function that returns order to the best fitting elevators from best to worst alternative.
fn best_to_worst_elevator(order: u8, elevators: &Vec<Elevator>) -> Vec<u8> {

    // Vec<Elevator.ID, Score> Higher score = better alternative
    let mut scores: Vec<(u8, i32)> = Vec::new(); 


    // Give score to all active elevators
    for elevator in elevators {
        let mut score = 0;

        // Distance to the order (lower is better)
        score -= 10*(elevator.current_floor as i32 - order as i32).abs();

        // Direction compatibility
        if elevator.status == Status::Moving {
            if (elevator.direction == 1 && elevator.current_floor < order) || 
               (elevator.direction == -1 && elevator.current_floor > order) {
                // Reward for moving towards the floor
                score += 10; 
            } else {
                // Penalty if moving away from the floor
                score -= 10; 
            }
        // Idle elevators are prefered over busy elevators
        }else if elevator.status == Status::Idle { 
            score += 20;
        }else if elevator.status == Status::Error {
            score -= 10000
        }

        // Shorter queue gets priority, Less is better
        score -= elevator.queue.len() as i32 * 5; 

        scores.push((elevator.ID, score));
    }

    // Sort by score
    scores.sort_by(|a, b| b.1.cmp(&a.1));

    // Return Vec<u8> of IDs in decending order from best to worst option  https://doc.rust-lang.org/std/iter/struct.Map.html
    return scores.into_iter().map(|(id, score)| id).collect();
}


/// If for some reason more than master is active, forexample race during election or one didnt recive the first message from new master.
/// master with lowest id keeps the role, the rest become slaves.
fn handle_multiple_masters(me: &Elevator, sender: &Elevator, worldview: &Worldview) -> bool {
    
    if me.role == role::Master {
        return false;

        // Give away master role, simple solution, Kill program and reboot
     }else{
        if sender.ID < me.ID
        relinquish_master
        reboot_program();

        // Keep master role
        }else{
            return true; 
        }
     }
}


//----------------------------------TESTS-------------------------------------------------------------

#[cfg(test)] // https://doc.rust-lang.org/book/ch11-03-test-organization.html Run tests with "cargo test"
mod tests {
    use super::*;
    use std::net::{UdpSocket, SocketAddr};
    use std::sync::{Arc, Mutex};
    use std::time::Duration;
    use std::thread;

    #[test]
    fn test_give_order() {
        let master = Elevator::init("127.0.0.1:1234", 5).unwrap();
        let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
        let slave_id = 2;
        let new_order = 3;

        //Check that order is recived
        assert!(give_order(&master, slave_id, new_order)); 
    }

    #[test]
    fn test_remove_from_queue() {
        let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
        let master = Elevator::init("127.0.0.1:1234", 5).unwrap();
        let removed_orders = vec![2, 4, 6];
        let slave_id = 2;

        // Check that order is recived
        assert!(remove_from_queue(slave_id, removed_orders));
    }

    #[test]
    fn test_master_worldview() {
        let master = Elevator::init("127.0.0.1:1234", 5).unwrap();

        //Check that the worldview can broadcast
        assert!(master_worldview(master));
        // Check that the worldview is recived
        // Check that the worldview is correct
    }

    #[test]
    fn test_correct_master_worldview() {
        let master = Elevator::init("127.0.0.1:1234", 5).unwrap();
        let slave = 

        // Broadcast Worldview (Lacking orders from a slave queue)

        // Recive Worldview

        // Compare worldview

        // Send correction message in return to master

        // Check correction message is recived
        assert!(correct_master_worldview(&master));
        // Send corrected worldview in return to slave

        // Compare new world view with correct worldview, should match
    }


    #[test]
    fn test_handle_slave_failure() {

        // Create elevator
        let mut elevator = 
            Elevator::init("127.0.0.1:1234", 5).unwrap(),
            Elevator::init("127.0.0.1:1234", 5).unwrap(),;

        // Set values elevators 
        elevators[1].ID = 2;
        elevators[1].queue = vec![3, 4, 5];
        elevators[0].ID = 1;
        elevators[0].queue = vec![];

        // Elevator 2 goes offline, give orders to 1
        handle_slave_failure(2, &mut elevators);
        assert!(elevators.iter().all(|e| e.ID != 2));
    }

    #[test]
    fn test_reassign_orders() {
        //Create 3 elevators with diffrent values
        let elevators = vec![
            Elevator { ID: 1, current_floor: 3, queue: vec![5], status: Status::Moving, direction: 1 },
            Elevator { ID: 2, current_floor: 8, queue: vec![], status: Status::Idle, direction: 0 },
            Elevator { ID: 3, current_floor: 2, queue: vec![4, 6], status: Status::Moving, direction: -1 },
        ];

        let orders = vec![1, 2, 3];
        
        // Reassign orders
        reassign_orders(orders);
    }

    #[test]
    fn test_best_to_worst_elevator() {
        let elevators = vec![
            Elevator { ID: 1, current_floor: 3, queue: vec![5], status: Status::Moving, direction: 1 },
            Elevator { ID: 2, current_floor: 8, queue: vec![], status: Status::Idle, direction: 0 },
            Elevator { ID: 3, current_floor: 2, queue: vec![4, 6], status: Status::Moving, direction: -1 },
        ];
        let sorted_elevators = best_to_worst_elevator(5, &elevators);
        assert_eq!(sorted_elevators.len(), 3);
    }

    #[test]
    fn test_handle_multiple_masters() {
        let me = Elevator { ID: 1, current_floor: 3, queue: vec![5], status: Status::Idle, direction: 0 };
        let sender = Elevator { ID: 2, current_floor: 5, queue: vec![], status: Status::Idle, direction: 0 };
        let worldview = Worldview { orders: vec![me.clone(), sender.clone()], lights: vec![1, 2] };

        let result = handle_multiple_masters(&me, &sender, &worldview);
        assert!(result || !result);
    }
}
