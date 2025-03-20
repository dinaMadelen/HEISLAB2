
//----------------------------------TESTS-------------------------------------------------------------

#[cfg(test)] // https://doc.rust-lang.org/book/ch11-03-test-organization.html Run tests with "cargo test"
mod tests {
    use super::*;
    use std::net::{UdpSocket, SocketAddr};
    use std::sync::{Arc, Mutex};
    use std::thread;
    use std::time::{Duration, Instant};

    /// Helper function to create a test UdpSocket
    fn create_test_socket() -> UdpSocket {
        UdpSocket::bind("127.0.0.1:0").expect("Failed to bind test UDP socket")
    }

    /// Helper function to create a test Elevator
    fn create_test_elevator() -> Elevator {
        Elevator::new(1) // Assuming `Elevator::new(id)` initializes an elevator with ID
    }

    #[test]
    fn test_receive_order() {
        let mut elevator = create_test_elevator();
        let socket = create_test_socket();
        let sender_address = "127.0.0.1:4000".parse().unwrap();
        let original_msg = UdpMsg::new(1, MessageType::New_Order, vec![3]);

        assert!(receive_order(&mut elevator, 3, &socket, sender_address, &original_msg));
        assert!(elevator.queue.contains(&3));

        // Should not add a duplicate order
        assert!(!receive_order(&mut elevator, 3, &socket, sender_address, &original_msg));
    }

    #[test]
    fn test_notify_completed() {
        let socket = create_test_socket();
        let slave_id = 1;
        let order = 2;

        // Just test that it does not panic
        notify_completed(slave_id, order);
    }

    #[test]
    fn test_cancel_order() {
        let mut elevator = create_test_elevator();
        elevator.queue.push(4);
        assert!(cancel_order(&mut elevator, 4));
        assert!(!elevator.queue.contains(&4));

        // Cancel a non-existent order
        assert!(!cancel_order(&mut elevator, 5));
    }

    #[test]
    fn test_update_from_worldview() {
        let mut elevator = create_test_elevator();
        elevator.queue.push(1);
        let new_worldview = vec![vec![1, 2, 3]];

        assert!(!update_from_worldview(&mut elevator, new_worldview.clone()));
        elevator.queue = vec![1, 2, 3];
        assert!(update_from_worldview(&mut elevator, new_worldview));
    }

    #[test]
    fn test_notify_worldview_error() {
        let socket = create_test_socket();
        let slave_id = 1;
        let missing_orders = vec![3, 4];
        notify_worldview_error(slave_id, &missing_orders);
    }

    #[test]
    fn test_check_master_failure() {
        let mut elevator = create_test_elevator();
        let last_lifesign_master = Instant::now();

        // Should return false since master is alive
        assert!(!check_master_failure(last_lifesign_master, &mut elevator));

        // Simulate expired master heartbeat
        let expired_time = Instant::now() - Duration::from_secs(6);
        assert!(check_master_failure(expired_time, &mut elevator));
    }

    #[test]
    fn test_set_new_master() {
        let mut elevator = create_test_elevator();

        // Before assuming master role
        assert!(matches!(elevator.role, Role::Slave));

        // Set new master
        set_new_master(&mut elevator);

        // After assuming master role
        assert!(matches!(elevator.role, Role::Master));
    }

    #[test]
    fn test_reboot_program() {
        // Not sure how to test this as it reboots the program, maybe drop it and just test while running the program?
        !todo("Figure out a way to test reboot_program");
    }

}


