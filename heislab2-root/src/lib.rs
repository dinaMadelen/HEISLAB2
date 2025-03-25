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

    pub mod cab_object{
        pub mod cab;
        pub mod elevator_movement;
        pub mod elevator_queue_handling;
        pub mod elevator_light_function;
        pub mod elevator_status_functions;
    }

    pub mod system_status;
    
    pub mod master_functions{
        pub mod master;
        pub mod master_test;
    }

    pub mod slave_functions{
        pub mod slave;
        pub mod slave_test;
    }

    pub mod udp_functions{
        pub mod udp;
        pub mod udp_test;
    }
    
    pub mod system_init;

    pub mod io {
        pub mod io_init;
        pub mod io_handle;
    }

}