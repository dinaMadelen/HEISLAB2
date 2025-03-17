# HEISLAB2
Heislab 2 

FULL IDEA:
Make a master-slave system where the master is chosen based on lowest id. Acs and nacs are implementet through udp, and in case of death of the master/slave (No reply for a certain period of time) the master/ will be reelected/reimplemented in the same manner as on initialization, and new queues will be distributed.

Every message contains ID, which makes all elevators aware of who is supposed to be Master. 

Threads that will be active:
    There will be one active thread for each elevator alive. This thread makes the single elevator function normally, send udp messages, and includes an udp_recieve_handle function, which will make the elevator update itself and broadcast according to wished behaviour. 

MODULES:
elevator_object{
    alias_lib:
    elevator_init:
    elevator_movement:
    elevator_queue_handling:
    elevator_status_funvtions:
    elevator_test:
    poll:
}

udp{
    udp_test:
    udp{
        "Objects"
        {
            message_type Enum:
            {
                Wordview,
                Ack,
                Nak,
                New_Order,
                New_Master,
                New_Online,
                Request_Queue,
                Respond_Queue,
                Error_Worldview,
                Error_Offline,
                Request_Resend,
            }
            UdpHeader Struct:
            {
                sender_id: u8,            // ID of the sender of the message.
                message_id: message_type, // ID for what kind of message it is, e.g. Button press, or Update queue.
                checksum: Vec<u8>,        // Hash of data to check message integrity.
            }

            UdpMessage Struct:
            {
                header: UdpHeader, // Header struct containing information about the message itself
                data: Vec<u8>,     // Data so be sent.
            }

        }

        Functions:
        {

        }
    }
}
master{
    master_test: Includes tests for master function. 
    master{
        "Objects"
        {
            Worldview Struct:
            Role Enum: 
        }

        Functions: give_order, remove_from_queue, correct_master_worldview, master_worldview, handle_slave_failure, reassign_orders, best_to_worst_elevator, handle_multiple_masters
        {
            give_order:
                Sends an order to a slave elevator and waits for acknowledgment.
                Uses UDP broadcasting with retries to ensure delivery.
                Returns true if the order is acknowledged, otherwise false.
                remove_from_queue

            remove_from_queue:
                Broadcasts a request to remove one or more orders from a specific elevator.
                Uses UDP messaging to ensure delivery.
                Returns true if acknowledged, otherwise false.
                correct_master_worldview

            correct_master_worldview:
                Compares received worldviews and merges them into a corrected version.
                Sends the updated worldview to all nodes.
                Returns true if successfully broadcasted.

            master_worldview:
                Broadcasts the current master worldview to all elevators.
                Returns true if successfully broadcasted.

            handle_slave_failure:
                Detects when a slave elevator fails and redistributes its orders.
                Removes the failed elevator from the active list.
                Returns true if orders were successfully reassigned, otherwise false.

            reassign_orders:
                Reassigns orders from a failed or unavailable elevator to active elevators.
                Uses best_to_worst_elevator to determine the best elevator for each order.
                Sends the reassigned orders using UDP messaging.

            best_to_worst_elevator: CAN BE REMADE TO COST FUNCTION
                Determines the best elevator to handle a given order based on a scoring system.
                Scores are based on distance, movement direction, queue length, and elevator status.
                Returns a sorted list of elevator IDs from best to worst.

            handle_multiple_masters
                Resolves conflicts when multiple elevators assume the master role.
                The elevator with the lowest ID becomes the master; others become slaves or reboot.
                Returns true if the elevator keeps the master role, otherwise false.
        }

    }
}

slave{
    slave_test: Includes tests fdor the slave functions.

    slave{
        Classes{
            Lifesign: Includes last lifesign timestamp from master.
        }

        Functions{
            receive_order:
                Receives an order from the master and adds it to the elevator’s queue if not already present. Sends an acknowledgment back to the master.
                Returns true if the order was added or already in the queue and acknowledged. false if acknowledgment failed.

            notify_completed:
                Broadcasts that an order has been completed.
                bool → true if the broadcast was successful, false if it failed.

            cancel_order:
                Removes an active order from the queue if it exists.
                bool → true if the order was successfully removed. false if the order was not found.

            update_from_worldview:
                Synchronizes the elevator’s queue with the master’s worldview and identifies missing orders.
                Arguments:
                active_elevators: &mut Vec<Elevator> → Reference to a list of active elevators.
                new_worldview: Vec<Vec<u8>> → Nested vector representing the master’s order worldview.
                bool → true if orders were successfully updated or already matched. false if there were missing orders.

            notify_worldview_error:
                Notifies the master that orders are missing from the worldview.
                slave_id: u8 → The ID of the elevator detecting the issue.
                missing_orders: Vec<u8> → A list of missing orders.
                (No return value, just sends a UDP message.)

            check_master_failure:
                Monitors the master's broadcast signal and initiates a master election if the master is unresponsive for more than 5 seconds.
                bool → true if the master was unresponsive and a new master election is started. false if the master is still active.

            set_new_master:
                Waits a calculated time before checking if the master role is taken. If not, assumes the master role and broadcasts the worldview.
                me: &mut Elevator → Reference to the elevator that may assume the master role.
                (No return value, but assigns a new master if needed.)

            reboot_program:
                Restarts the program by launching a new instance and terminating the current one.
        }
    }
}
