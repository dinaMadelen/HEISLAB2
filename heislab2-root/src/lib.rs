pub mod modules{
    pub mod elevator_object {
        pub mod elevator_init;
        pub mod poll;
        pub mod elevator_test;
        pub mod alias_lib;
    }
    
    pub mod order_object{
        pub mod order_init;
    }

    pub mod cab{
        pub mod cab;
        pub mod elevator_movement;
        pub mod elevator_queue_handling;
        pub mod elevator_light_function;
        pub mod elevator_status_functions;
    }

    pub mod system_status;
    
    pub mod master{
        pub mod master;
        pub mod master_test;
    }

    pub mod slave{
        pub mod slave;
        pub mod slave_test;
    }

    pub mod udp{
        pub mod udp;
        pub mod udp_test;
    }
    
    pub mod system_init;

    pub mod io {
        pub mod io_init;
    }

}