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

    pub mod master{
        pub mod master;
        pub mod master_test;
    };

    pub mod slave{
        pub mod slave;
        pub mod slave_test;
    };

    pub mod udp{
        pub mod udp;
        pub mod udp_test;
    };
    
    pub mod system_init;
    pub mod system_status;
    pub mod udp;
    pub mod cab;
    

    pub mod io {
        pub mod io_init;
    }

}