

//------------------------------Tests-----------------------

#[cfg(test)] // https://doc.rust-lang.orgbook/ch11-03-test-organization.html Run tests with "cargo test"
mod tests {
    use std::net::{UdpSocket, SocketAddr};
    use std::sync::{Arc, Mutex};
    use std::thread;
    use std::time::Duration;
    use crate::modules::udp::*;
    use udp::{MessageType, UdpHeader, UdpMsg};

    #[test]
    fn test_serialize_deserialize() {
        let msg = UdpMsg {
            header: UdpHeader {
                sender_id: 1,
                message_id: message_type::Ack,
                checksum: vec![0x12, 0x34],
            },
            data: vec![1, 2, 3, 4],
        };

        let serialized = serialize(&msg);
        let deserialized = deserialize(&serialized).expect("Deserialization failed");

        assert_eq!(msg.header.sender_id, deserialized.header.sender_id);
        assert_eq!(msg.header.message_id as u8, deserialized.header.message_id as u8);
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
                checksum: checksum.clone(),
            },
            data: data.clone(),
        };
        assert!(comp_checksum(&msg));
    }

    #[test]
    fn test_udp_send_receive() {
        let socket = UdpSocket::bind("127.0.0.1:0").expect("Failed to bind socket");
        let local_addr = socket.local_addr().expect("Failed to get socket address");

        let msg = UdpMsg {
            header: UdpHeader {
                sender_id: 1,
                message_id: message_type::Ack,
                checksum: calc_checksum(&vec![1, 2, 3, 4]),
            },
            data: vec![1, 2, 3, 4],
        };

        let send_socket = socket.try_clone().expect("Failed to clone socket");
        let recv_socket = socket;

        let msg_clone = msg.clone();
        thread::spawn(move || {
            thread::sleep(Duration::from_millis(50));
            udp_send(&send_socket, local_addr, &msg_clone);
        });

        let received_msg = udp_receive_ensure(&recv_socket, 5, 2).expect("Failed to receive message");
        assert_eq!(msg.data, received_msg.data);
    }

    #[test]
    fn test_udp_ack_nak() {
        let socket = UdpSocket::bind("127.0.0.1:0").expect("Failed to bind socket");
        let local_addr = socket.local_addr().expect("Failed to get socket address");

        let original_msg = UdpMsg {
            header: UdpHeader {
                sender_id: 1,
                message_id: message_type::New_Order,
                checksum: calc_checksum(&vec![5, 10, 15]),
            },
            data: vec![5, 10, 15],
        };

        let send_socket = socket.try_clone().expect("Failed to clone socket");
        let recv_socket = socket;

        thread::spawn(move || {
            thread::sleep(Duration::from_millis(50));
            udp_ack(&send_socket, local_addr, &original_msg, 2);
        });

        let received_ack = udp_receive_ensure(&recv_socket, 5, 2).expect("Failed to receive ACK");
        assert_eq!(received_ack.header.message_id, message_type::Ack);
        assert_eq!(received_ack.data, original_msg.header.checksum);
    }

    #[test]
    fn test_udp_send_ensure() {
        let socket = UdpSocket::bind("127.0.0.1:0").expect("Failed to bind socket");
        let local_addr = socket.local_addr().expect("Failed to get socket address");

        let msg = UdpMsg {
            header: UdpHeader {
                sender_id: 1,
                message_id: message_type::New_Order,
                checksum: calc_checksum(&vec![8, 16, 32]),
            },
            data: vec![8, 16, 32],
        };

        let mut sent_messages = Vec::new();

        let send_socket = socket.try_clone().expect("Failed to clone socket");
        let recv_socket = socket;

        let msg_clone = msg.clone();
        thread::spawn(move || {
            thread::sleep(Duration::from_millis(50)); 
            udp_ack(&send_socket, local_addr, &msg_clone, 2);
        });

        let result = udp_send_ensure(&recv_socket, &local_addr.to_string(), &msg, 3, &mut sent_messages);
        assert!(result);
    }

    #[test]
    fn test_handle_ack_nak_logic() {
        let mut sent_messages = Vec::new();
        let msg = UdpMsg {
            header: UdpHeader {
                sender_id: 1,
                message_id: message_type::New_Order,
                checksum: calc_checksum(&vec![2, 4, 6]),
            },
            data: vec![2, 4, 6],
        };

        sent_messages.push(msg.clone());

        let ack_msg = UdpMsg {
            header: UdpHeader {
                sender_id: 2,
                message_id: message_type::Ack,
                checksum: calc_checksum(&msg.data),
            },
            data: calc_checksum(&msg.data),
        };

        handle_ack(ack_msg, &mut sent_messages);
        assert!(!sent_messages.iter().any(|m| calc_checksum(&m.data) == ack_msg.data));
    }

    #[test]
    fn test_serialize_deserialize() {
        let msg = UdpMsg {
            header: UdpHeader {
                sender_id: 1,
                message_type:MessageType::Ack,
                checksum: vec![0x12, 0x34],
            },
            data: vec![1, 2, 3, 4],
        };

        let serialized = serialize(&msg);
        let deserialized = deserialize(&serialized).expect("Deserialization failed");

        assert_eq!(msg.header.sender_id, deserialized.header.sender_id);
        assert_eq!(msg.header.message_id as u8, deserialized.header.message_id as u8);
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
                message_type:MessageType::Ack,
                checksum: checksum.clone(),
            },
            data: data.clone(),
        };
        assert!(comp_checksum(&msg));
    }

    /* 
    #[test]
    fn test_udp_send_receive() {
        let socket = UdpSocket::bind("127.0.0.1:0").expect("Failed to bind socket");
        let local_addr = socket.local_addr().expect("Failed to get socket address");

        let msg = UdpMsg {
            header: UdpHeader {
                sender_id: 1,
                message_type:MessageType::Ack,
                checksum: calc_checksum(&vec![1, 2, 3, 4]),
            },
            data: vec![1, 2, 3, 4],
        };

        let send_socket = socket.try_clone().expect("Failed to clone socket");
        let recv_socket = socket;

        let msg_clone = msg.clone();
        thread::spawn(move || {
            thread::sleep(Duration::from_millis(50));
            udp_send(&send_socket, local_addr, &msg_clone);
        });

        let received_msg = udp_receive_ensure(&recv_socket, 5, 2).expect("Failed to receive message");
        assert_eq!(msg.data, received_msg.data);
    }
    */

    #[test]
    fn test_udp_ack_nak() {
        let socket = UdpSocket::bind("127.0.0.1:0").expect("Failed to bind socket");
        let local_addr = socket.local_addr().expect("Failed to get socket address");

        let original_msg = UdpMsg {
            header: UdpHeader {
                sender_id: 1,
                message_type:MessageType::New_Order,
                checksum: calc_checksum(&vec![5, 10, 15]),
            },
            data: vec![5, 10, 15],
        };

        let send_socket = socket.try_clone().expect("Failed to clone socket");
        let recv_socket = socket;

        thread::spawn(move || {
            thread::sleep(Duration::from_millis(50));
            udp_ack(&send_socket, local_addr, &original_msg, 2);
        });

        let received_ack = udp_receive_ensure(&recv_socket, 5, 2).expect("Failed to receive ACK");
        assert_eq!(received_ack.header.message_id, MessageType::Ack);
        assert_eq!(received_ack.data, original_msg.header.checksum);
    }

    /* 
    #[test]
    fn test_udp_send_ensure() {
        let socket = UdpSocket::bind("127.0.0.1:0").expect("Failed to bind socket");
        let local_addr = socket.local_addr().expect("Failed to get socket address");

        let msg = UdpMsg {
            header: UdpHeader {
                sender_id: 1,
                message_type:MessageType::New_Order,
                checksum: calc_checksum(&vec![8, 16, 32]),
            },
            data: vec![8, 16, 32],
        };

        let mut sent_messages = Vec::new();

        let send_socket = socket.try_clone().expect("Failed to clone socket");
        let recv_socket = socket;

        let msg_clone = msg.clone();
        thread::spawn(move || {
            thread::sleep(Duration::from_millis(50)); 
            udp_ack(&send_socket, local_addr, &msg_clone, 2);
        });

        let result = udp_send_ensure(&recv_socket, &local_addr.to_string(), &msg, 3, &mut sent_messages);
        assert!(result);
    }
    */


    #[test]
    fn test_handle_ack_nak_logic() {
        let mut sent_messages = Vec::new();
        let msg = UdpMsg {
            header: UdpHeader {
                sender_id: 1,
                message_type:MessageType::New_Order,
                checksum: calc_checksum(&vec![2, 4, 6]),
            },
            data: vec![2, 4, 6],
        };

        sent_messages.push(msg.clone());

        let ack_msg = UdpMsg {
            header: UdpHeader {
                sender_id: 2,
                message_type:MessageType::Ack,
                checksum: calc_checksum(&msg.data),
            },
            data: calc_checksum(&msg.data),
        };

        handle_ack(ack_msg, &mut sent_messages);
        assert!(!sent_messages.iter().any(|m| calc_checksum(&m.data) == ack_msg.data));
    }
}

