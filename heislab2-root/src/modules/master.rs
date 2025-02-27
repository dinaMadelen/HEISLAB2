

/*Fordeling av ordre
fordeling av ordre basert på kost funksjon

Worldview broadcast & retting av feil i worldview
sending av wordview(eksisterer broadcast funksjon i UDP modulen)

Gi fra seg master rolle
usikker på om denne trengs da slaver skal ta over rollen automatisk hvis master går offline

Styring av lys
kontroll å slå på lys når det er bekreftet at alle har mottatt(må bære bekreftet for å sikre inte)

Example for worldview vector
worldview: Vec<Vec<u8>> = [[slave1.id,order1,order2,order3,order4],
                          [slave2.id,order1,order2],
                          [slave3.id,order1,order2,order3],
                          [slave4.id,order1,order2,order3,order4,order5]]

*/

//Broadcast order and wait for responce from reciver, if not recived resend, if this fail. find return false
//possibly not needed as each elevator will add any new order in the worldview to the queue.
// could just ack the broadcast form the slave that gets assigned the order
fn give_order(master: &Elevator, slave_id: u8, new_order: u8) -> bool {

    let mut retries = 3;
    let message = make_udp_msg(master.id, MessageType::NewOrder, new_order);

    while retries > 0 {
        if udp_send_ensure(&socket, &slave_address(slave_id), &message) {
            return true;
        }
        retries -= 1;
    }

    println!("Failed to deliver order to slave {}", slave_id);
    return false;
}


// Send order to remove a or more orders from a specific elevator
fn remove_from_queue(slave_id: u8, removed_orders: Vec<u8>) -> bool {

    let message = make_udp_msg(master.id, MessageType::RemoveOrder, removed_orders);
    return udp_send_ensure(&socket, &slave_address(slave_id), &message);
}

// Compare message and send out union of the recived and current worldview
fn correct_master_worldview(master: &Elevator) -> bool {

    let state = collect_system_state();
    let message = make_udp_msg(master.id, MessageType::Worldview, state);
    
    return udp_broadcast(&socket, &message);
}

// Broadcast worldview
fn master_worldview(master:Elevator) -> bool{
    
    make_Udp_msg(sender_id: master,message_type: Wordview, message:Vec<u8>) 
    
    return udp_broadcast(&socket,&message);
}

// Give away master role
fn relinquish_master(master: &mut Elevator) -> bool {

    let message = make_udp_msg(master.id, MessageType::RelinquishMaster, vec![]);
    udp_broadcast(&socket, &message);

    master.status = Elevator.status::Error;
    return true;
}

// Handle slave error
fn handle_slave_failure(slave_id: u8) {
    println!("Slave {} is not responding, redistributing orders.", slave_id);
    
    let orders = retrieve_pending_orders(slave_id);
    for order in orders {
        assign_order_to_best_elevator(order);
    }
}

// Reassign order
fn reassign_orders(orders: Vec<u8>) {
    for order in orders {
        assign_order_to_best_elevator(order);
    }
}

// Cost function that assigns order to the best fitting elevator
fn assign_order_to_best_elevator(order:u8){
todo!("Cost function that find best fitting elevator")

}
// If for some reason more than master is active, forexample race during election
// master with lowest id keeps the role, the rest swap to slaves.
fn handle_multiple_masters(){
    todo!("handle case where two or more masters are active at the same time")
}



