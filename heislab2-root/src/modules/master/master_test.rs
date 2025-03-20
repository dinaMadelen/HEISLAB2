

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
            Elevator::init("127.0.0.1:1234", 5).unwrap();

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
