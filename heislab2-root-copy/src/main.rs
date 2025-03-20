use std::thread::*;
use std::time::*;
use crossbeam_channel as cbc;
use std::env;
use std::net;
use std::process;



use heislab2_root_test::modules::elevator_object::*;
use alias_lib::{ DIRN_DOWN, DIRN_STOP};
use elevator_init::Elevator;
use elevator_status_functions::Status;
use heislab2_root_test::modules::order_object::order_init::Order;
//use master::master::*;
// slave::slave::*;
//use udp::udp::*;
//use udp::message_type;

use heislab2_root_test::modules::udpnet;

// Data types to be sent on the network must derive traits for serialization
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
struct CustomDataType {
    message: String,
    iteration: u64,
}


// THIS IS SUPPOSED TO BE A SINGLE ELEVATOR MAIN THAT CAN RUN IN ONE THREAD


fn main() -> std::io::Result<()> {
    let elev_num_floors = 4;
    let mut elevator = Elevator::init("localhost:15657", elev_num_floors)?;
    println!("Elevator started:\n{:#?}", elevator);

    let poll_period = Duration::from_millis(25);

    let (call_button_tx, call_button_rx) = cbc::unbounded::<poll::CallButton>();
    {
        let elevator = elevator.clone();
        spawn(move || poll::call_buttons(elevator, call_button_tx, poll_period));
    }

    let (floor_sensor_tx, floor_sensor_rx) = cbc::unbounded::<u8>();
    {
        let elevator = elevator.clone();
        spawn(move || poll::floor_sensor(elevator, floor_sensor_tx, poll_period));
    }

    let (stop_button_tx, stop_button_rx) = cbc::unbounded::<bool>();
    {
        let elevator = elevator.clone();
        spawn(move || poll::stop_button(elevator, stop_button_tx, poll_period));
    }

    let (obstruction_tx, obstruction_rx) = cbc::unbounded::<bool>();
    {
        let elevator = elevator.clone();
        spawn(move || poll::obstruction(elevator, obstruction_tx, poll_period));
    }

    let args: Vec<String> = env::args().collect();
    let id = if args.len() > 1 {
        args[1].clone()
    } else {
        let local_ip = net::TcpStream::connect("8.8.8.8:53")
            .unwrap()
            .local_addr()
            .unwrap()
            .ip();
        format!("rust@{}#{}", local_ip, process::id())
    };

    let msg_port = 19735;
    let peer_port = 19738;

    // The sender for peer discovery
    let (peer_tx_enable_tx, peer_tx_enable_rx) = cbc::unbounded::<bool>();
    let _handler = {
        let id = id.clone();
        spawn(move || {
            if udpnet::peers::tx(peer_port, id, peer_tx_enable_rx).is_err() {
                // crash program if creating the socket fails (`peers:tx` will always block if the
                // initialization succeeds)
                process::exit(1);
            }
        })
    };

    // (periodically disable/enable the peer broadcast, to provoke new peer / peer loss messages)
    // This is only for demonstration purposes, if using this module in your project do not include
    // this
    spawn(move || loop {
        sleep(Duration::new(6, 0));
        peer_tx_enable_tx.send(false).unwrap();
        sleep(Duration::new(3, 0));
        peer_tx_enable_tx.send(true).unwrap();
    });

    // The receiver for peer discovery updates
    let (peer_update_tx, peer_update_rx) = cbc::unbounded::<udpnet::peers::PeerUpdate>();
    {
        spawn(move || {
            if udpnet::peers::rx(peer_port, peer_update_tx).is_err() {
                // crash program if creating the socket fails (`peers:rx` will always block if the
                // initialization succeeds)
                process::exit(1);
            }
        });
    }

    // Periodically produce a custom data message
    let (custom_data_send_tx, custom_data_send_rx) = cbc::unbounded::<CustomDataType>();
    {/*
        spawn(move || {
            //DEFINES A MESSAGE
            let mut cd = CustomDataType {
                message: format!("Hello from node {}", id),
                iteration: 0,
            };
            loop {
                custom_data_send_tx.send(cd.clone()).unwrap();
                cd.iteration += 1;
                sleep(Duration::new(1, 0));
            }
        });
        */
    }
    // The sender for our custom data
    {
        spawn(move || {
            if udpnet::bcast::tx(msg_port, custom_data_send_rx).is_err() {
                // crash program if creating the socket fails (`bcast:tx` will always block if the
                // initialization succeeds)
                process::exit(1);
            }
        });
    }
    // The receiver for our custom data
    let (custom_data_recv_tx, custom_data_recv_rx) = cbc::unbounded::<CustomDataType>();
    spawn(move || {
        if udpnet::bcast::rx(msg_port, custom_data_recv_tx).is_err() {
            // crash program if creating the socket fails (`bcast:rx` will always block if the
            // initialization succeeds)
            process::exit(1);
        }
    });

    
    let dirn = DIRN_DOWN;


    if elevator.floor_sensor().is_none() {
        elevator.motor_direction(dirn);
    }
    
    let (door_tx, door_rx) = cbc::unbounded::<bool>();

    loop {
        cbc::select! {
            //tror denne kan bli
            recv(door_rx) -> a => {
                let door_signal = a.unwrap();
                if door_signal {
                    elevator.go_next_floor(door_tx.clone(),obstruction_rx.clone());
                }
            }

            recv(call_button_rx) -> a => {
                let call_button = a.unwrap();
                println!("{:#?}", call_button);
                //Make new order and add that order to elevators queue
                let new_order = Order::init(call_button.floor,call_button.call);
                
                //broadcast addition, but since order is in own cab the others taking over will not help
                /*
                make_Udp_msg(elevator, message_type::New_Order); //Broadcasts the new order so others can update wv

                if call_button.call == CAB{
                    elevator.add_to_queue(new_order);
                }
                */
                elevator.add_to_queue(new_order);
                elevator.turn_on_queue_lights();

                //Safety if elevator is idle to double check if its going to correct floor
                let true_status= elevator.status.lock().unwrap();
                let clone_true_status = true_status.clone();
                drop(true_status);

                if clone_true_status == Status::Idle{
                    elevator.go_next_floor(door_tx.clone(),obstruction_rx.clone());
                }

                
            },

            recv(floor_sensor_rx) -> a => {
                let floor = a.unwrap();
                println!("Floor: {:#?}", floor);

                //update current floor status
                elevator.current_floor = floor;
                /*
                make_Udp_msg(elevator,message_type::Wordview) //guess this is the ping form
                */
                
                //keep following current route
                elevator.go_next_floor(door_tx.clone(),obstruction_rx.clone());
                
            },

            /*Burde nok modifiseres*/
            recv(stop_button_rx) -> a => {
                let stop = a.unwrap();
                println!("Stop button: {:#?}", stop);
                
                elevator.set_status(Status::Stop);

                //broadcast current floor, stop and current queue - this might be redistributed

                //turn of lights
                for f in 0..elev_num_floors {
                    for c in 0..3 {
                        elevator.call_button_light(f, c, false);
                    }
                }
                

            },
            recv(obstruction_rx) -> a => {
                let obstr = a.unwrap();
                println!("Obstruction: {:#?}", obstr);
                elevator.motor_direction(if obstr { DIRN_STOP } else { dirn });

                //broadcast obstruction
            },

            recv(peer_update_rx) -> a => {
                let update = a.unwrap();
                println!("{:#?}", update);
            }
            recv(custom_data_recv_rx) -> a => {
                let cd = a.unwrap();
                println!("{:#?}", cd);

            }
            //recv UDP message

            //check message type 
            //if message is from master
            // MAKE HANDLE MASTER MESSAGE FUNCTION
            

            //if order is yours
            //add to own queue

            //if order is someone elses
            //add to full queue

            //if message is from slave 
            //if order, add to own full queue and world view
            //if message is an ack update elevators alive
            
            
        }
    }
}
