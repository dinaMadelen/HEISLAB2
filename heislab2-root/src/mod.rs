pub mod modules{
    pub mod elevator_object {
        pub mod elevator_init;
        pub mod elevator_movement;
        pub mod elevator_queue_handling;
        pub mod poll;
        pub mod elevator_test;
        pub mod elevator_status_functions;
        pub mod alias_lib;
    }
    
    pub mod order_object{
        pub mod order_init;
    }

    pub mod master;
    pub mod slave;
    pub mod system_init;
    pub mod system_status;
    pub mod udp;
    
}
