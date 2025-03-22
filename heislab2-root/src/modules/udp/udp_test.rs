/* 
//------------------------------Tests-----------------------

#[cfg(test)] // https://doc.rust-lang.orgbook/ch11-03-test-organization.html Run tests with "cargo test"
mod tests {
    use std::net::{UdpSocket, SocketAddr};
    use std::sync::{Arc, Mutex};
    use std::thread;
    use std::time::Duration;

    use crate::modules::system_status::SystemState;
    use crate::modules::master::master::Role;

    use crate::modules::udp::udp::*;
    

    #[test]
    fn test_make_udpmsg(){

    // Create a test order
    let test_order = Order { floor: 2, order_type: CAB };
    let data = UdpData::Order(test_order.clone());

    // Generate a UDP message
    let sender_id = 1
    let msg_type = MessageType::NewRequest;
    let udp_msg = make_udp_msg(sender_id, msg_type.clone(), data.clone());

    // Check header fields
    assert_eq!(udp_msg.header.sender_id, sender_id);
    assert_eq!(udp_msg.header.message_type, msg_type);

    // Check checksum is correctly calculated
    let expected_checksum = calc_checksum(&data);
    assert_eq!(udp_msg.header.checksum, expected_checksum);

    // Check data integrity
    assert_eq!(udp_msg.data, data);

    }

    #[test]
    fn test_serialize_deserialize() {

        //Create test order
        let order = Order{
            floor: 3,
            order_type:1
        };

        // Clone to UdpData 
        data = UdpData::Order(order.clone());

        //Create test mesage
        let msg = UdpMsg {
            header: UdpHeader {
                sender_id: 1,
                message_type: MessageType::NewRequest,
                checksum: calc_checksum(&data),
            },
            data: data.clone(),
        };

        // Serialize and deserialize
        let serialized = msg_serialize(&msg);
        let deserialized = msg_deserialize(&serialized).expect("Deserialization failed");

        // Checks
        assert_eq!(msg.header.sender_id, deserialized.header.sender_id);
        assert_eq!(msg.header.message_type, deserialized.header.message_type);
        assert_eq!(msg.data, deserialized.data);
        assert!(comp_checksum(&deserialized));
    }


    #[test]
    fn test_udp_send_receive() {

        //Create two sockets on localhost
        let localhost = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
        let recv_addr: SocketAddr = SocketAddr::new(localhost, 3500);;
        let send_addr: SocketAddr = SocketAddr::new(localhost, 3600);;
        
        //Create reaciving cab
        let recv_cab = Cab {
            inn_address: recv_addr,
            out_address: send_addr,
            num_floors: 4,
            id: 2,
            current_floor: 0,
            queue: vec![],
            status: Status::Idle,
            direction: 0,
            role: Role::Slave,
        };
        
        //Create sending cab
        let send_cab = Cab {
            inn_address: send_addr,
            out_address: recv_addr,
            num_floors: 4,
            id: 1,
            current_floor: 0,
            queue: vec![],
            status: Status::Idle,
            direction: 0,
            role: Role::Slave,
        };
    
        //Create udp handlers for the cabs
        let sender = init_udp_handler(send_cab);
        let receiver = init_udp_handler(recv_cab);
    
        //Create test message
        let test_order = Order { floor: 3, order_type: CAB };
        let msg = make_udp_msg(1, MessageType::NewRequest, UdpData::Order(test_order.clone()));
    
        // Dummy system state
        let mut dummy_state = SystemState {
            me_id: 3,
            master_id: Arc::new(Mutex::new(1)),
            last_lifesign: Arc::new(Mutex::new(std::time::Instant::now())),
            last_worldview: Arc::new(Mutex::new(msg.clone())),
            active_elevators: Arc::new(Mutex::new(vec![])),
            failed_orders: Arc::new(Mutex::new(vec![])),
            sent_messages: Arc::new(Mutex::new(vec![])),
        };
    
        // Spawn a thread to simulate sending
        let sender_clone = sender;
        let msg_clone = msg.clone();
        thread::spawn(move || {
            //Delay to ensure that reciuver is up and running
            thread::sleep(Duration::from_millis(100));
            sender_clone.send(&recv_addr, &msg_clone);
        });
    
        // Try receiving on the other handler
        let received = receiver.receive(100, &mut dummy_state).expect("No message received");

        //Handle messagetype?
    
        //Assert it's the same message
        assert_eq!(received.header.sender_id, msg.header.sender_id);
        assert_eq!(received.header.message_type, msg.header.message_type);
        assert_eq!(received.data, msg.data);
    }

    #[test]
    fn test_udp_ack_nak() {

        //Create two sockets on localhost
        let recv_addr: SocketAddr = "127.0.0.1:3500".parse().unwrap();
        let send_addr: SocketAddr = "127.0.0.1:3600".parse().unwrap();
        
        //Create reaciving cab
        let recv_cab = Cab {
            inn_address: recv_addr,
            out_address: send_addr,
            num_floors: 4,
            id: 2,
            current_floor: 0,
            queue: vec![],
            status: Status::Idle,
            direction: 0,
            role: Role::Slave,
        };
        
        //Create sending cab
        let send_cab = Cab {
            inn_address: send_addr,
            out_address: recv_addr,
            num_floors: 4,
            id: 1,
            current_floor: 0,
            queue: vec![],
            status: Status::Idle,
            direction: 0,
            role: Role::Slave,
        };
    
        //Create udp handlers for the cabs
        let sender = init_udp_handler(send_cab);
        let receiver = init_udp_handler(recv_cab);

        // Create dummy message that should be responded to
        let test_order = Order { floor: 1, order_type: CAB };
        let data = UdpData::Order(test_order.clone());
        let original_msg = make_udp_msg(1, MessageType::NewRequest, data.clone());

        // Send ACK from sender to receiver
        let msg_clone = original_msg.clone();
        thread::spawn(move || {
            thread::sleep(Duration::from_millis(50));
            udp_ack(receiver_addr, &msg_clone, 2, &sender_clone);
        });

    // Dummy system state for receiver
    let mut dummy_state = SystemState {
        me_id: 2,
        master_id: Arc::new(Mutex::new(1)),
        last_lifesign: Arc::new(Mutex::new(std::time::Instant::now())),
        last_worldview: Arc::new(Mutex::new(original_msg.clone())),
        active_elevators: Arc::new(Mutex::new(vec![])),
        failed_orders: Arc::new(Mutex::new(vec![])),
        sent_messages: Arc::new(Mutex::new(vec![])),
    };

    // Receive the ACK
    let received = receiver.receive(100, &mut dummy_state).expect("Did not receive ACK");

    assert_eq!(received.header.message_type, MessageType::Ack);
    assert_eq!(received.header.checksum, calc_checksum(&data));
    assert_eq!(received.data, UdpData::None);
    }

}

#[test]
fn test_udp_msg_size() {

// Create some dummy elevators

let localhost = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
let dummy_elevators: Vec<Cab> = (0..5)
    .map(|id| Cab {
    id,
    current_floor: id,
    direction: 0,
    status: Status::Idle,
    num_floors: 4,
    queue: vec![
        Order { floor: 0, order_type: CAB },
        Order { floor: 1, order_type: CAB },
        ],
    role: Role::Slave,
    inn_address: SocketAddr::new(localhost, 3500),
    out_address: SocketAddr::new(localhost, 3600),
    })
    .collect();

    let worldview_msg = make_udp_msg(42, MessageType::Worldview, UdpData::Cabs(dummy_elevators));
    let serialized = msg_serialize(&worldview_msg);
    let size = serialized.len();

    println!("Serialized worldview message size: {} bytes", size);

    // Check it's under n bytes (We should check how much we are allowed to send ) or your safe UDP threshold
    // On Linux/Mac    ping -M do -s 1472 <Some ip on the network>
    // On Windows      ping ping -f -l 1472 <Some ip on the network>
    assert!(size < 1024, "Worldview message too large: {} bytes", size);
}

 */