#[cfg(test)] // https://doc.rust-lang.org/book/ch11-03-test-organization.html Run tests with "cargo test"
mod tests {
    use super::*;
    use std::net::UdpSocket;

    #[test]
    fn test_set_status() {
        let elev_num_floors = 4;
        let mut elevator = e::Elevator::init("localhost:15657", elev_num_floors).expect("Failed to initialize elevator");

        elevator.set_status(Status::Idle);
        assert_eq!(elevator.status, Status::Idle);

        elevator.set_status(Status::Moving);

    }


    #[test]
    fn test_serialize_deserialize() {
        let msg = UdpMsg {
            header: UdpHeader {
                sender_id: 1,
                message_id: message_type::Ack,
                sequence_number: 0,
                checksum: vec![0x12, 0x34],
            },
            data: vec![1, 2, 3, 4],
        };

        let serialized = serialize(&msg);
        let deserialized = deserialize(&serialized).expect("Deserialization failed");

        assert_eq!(msg.header.sender_id, deserialized.header.sender_id);
        assert_eq!(msg.header.sequence_number, deserialized.header.sequence_number);
        assert_eq!(msg.data, deserialized.data);
    }

    #[test]
    fn test_calc_checksum() {
        let data = vec![1, 2, 3, 4];
        let checksum = calc_checksum(&data);
        assert!(!checksum.is_empty());
    }

    #[test]
    fn test_comp_checksum() {
        let data = vec![1, 2, 3, 4];
        let checksum = calc_checksum(&data);
        let msg = UdpMsg {
            header: UdpHeader {
                sender_id: 1,
                message_id: message_type::Ack,
                sequence_number: 0,
                checksum,
            },
            data,
        };
        assert!(comp_checksum(&msg));
    }

    #[test]
    fn test_udp_send_recv() {
        let socket = UdpSocket::bind("127.0.0.1:0").expect("Failed to bind socket");
        let local_addr = socket.local_addr().expect("Failed to get socket address");
            
        let msg = UdpMsg {
            header: UdpHeader {
                sender_id: 1,
                message_id: message_type::Ack,
                sequence_number: 0,
                checksum: vec![0x12, 0x34],
            },
            data: vec![1, 2, 3, 4],
        };
            
        let send_socket = socket.try_clone().expect("Failed to clone socket");
        let recv_socket = socket;
            
        let msg_clone = msg.clone();
        std::thread::spawn(move || {
            //delay?
            udp_send(&send_socket, local_addr, &msg_clone);
        });
            
        let received_msg = udp_recive(&recv_socket, 5).expect("Failed to receive message");
        assert_eq!(msg.data, received_msg.data);
    }
}



