use std::io::{Read, Write};
use std::net::TcpStream;
use std::thread;
use std::time::Duration;
use crossbeam_channel::{Receiver, Sender};
//use serde::{Serialize, Deserialize};
use bincode;
use driver_rust::elevio::poll::CallButton;

//use crate::tcp_elevator;

//use tcp_elevator::elevator::{self, ButtonType};
//#[path = "tcp_elevator.rs"]
//mod tcp_elevator;
use crate::elevator::ButtonType;
use crate::tcp_definitions::{tcp_message, TcpLiftOrderT};

/// Handles TCP orders from the server/master.
/// Also forwards received call button presses to `main.rs` via `call_button_main_tx`,
/// ensuring that `main.rs` processes all button events.
pub fn receive_orders(server_address: &str, sender: Sender<tcp_message>, read_calls: Receiver<CallButton>, call_button_main_tx: Sender<CallButton>,elevator_id: u32 ) {
    
    
    loop {
        
        
        if let Ok(call_button) = read_calls.try_recv() {
           
            /* //for debugging
        let order = TCP_lift_order_t{
               floor: call_button.floor as u32,
               button_type:ButtonType::from_u8(call_button.call),
              elevator_id: elevator_id,
            };

            let message = tcp_message::set_order { order  };
            println!("button type: {:?}, order floor:{}, ID: {},", order.button_type, order.floor, order.elevator_id);
                */
            
            if let Err(e) = call_button_main_tx.send(call_button){println!("Error with sending to main: {}",e)};
        } 
        
        match TcpStream::connect(server_address) { 
            Ok(mut stream) => {
                println!("Connected to master at {}", server_address);

                let mut buffer = vec![0; 1024]; 

                loop { //samme stream, Må sende. 
                    if let Ok(call_button) = read_calls.try_recv() {

                        let order = TcpLiftOrderT{
                            floor: call_button.floor as u32,
                            button_type:ButtonType::from_u8(call_button.call),
                            elevator_id: elevator_id,
                        };
                         let message = tcp_message::set_order { order  };
                        let serialized_order = bincode::serialize(&message).expect("Serialization failed");

                        if let Err(e) = stream.write_all(&serialized_order) {
                            eprintln!("Failed to send local call to server: {}", e);
                        } else {
                            println!("Sent local call to server: {:?}", message);
                        }

                        if let Err(e) = call_button_main_tx.send(call_button){println!("Error with sending to main: {}",e)};

                        println!("order sendt to server: Floor: {}, Call: {:?}, ID: {},", order.floor, order.button_type, order.elevator_id);

                    }
                       
                    match stream.read(&mut buffer) {
                        Ok(0) => {
                            println!("Connection closed by server. Reconnecting...");
                            break;
                        }
                        Ok(n) => {
                            match bincode::deserialize::<tcp_message>(&buffer[..n]) {
                                Ok(tcp_message::set_order { order }) => {
                                    println!("Received order: Floor {}, Button {:?}", order.floor, order.button_type);
                                    sender.send(tcp_message::set_order { order }).expect("Failed to send order to elevator module");
                                }
                                Ok(tcp_message::clear_order { order }) => {
                                    println!("Clear order: Floor {}, Button {:?}", order.floor, order.button_type);
                                    sender.send(tcp_message::clear_order { order }).expect("Failed to send clear order to elevator module");
                                }
                                Ok(tcp_message::NOP { elevator_id }) => {
                                    println!("NOP received for elevator {}", elevator_id);
                                }
                                Err(e) => eprintln!("Deserialization error: {}", e),
                            }
                        }
                        Err(e) => {
                            println!("Failed to read from server: {}. Reconnecting...", e);
                            break;
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("Could not connect to master: {}. Retrying...", e);
                thread::sleep(Duration::from_secs(1));
            }
        }
    }
}

// BUG: creates a new tcp connection. the current server implementation does not like this.
// send_order and recieve_order should be made to share the tcp connection.
pub fn send_order(server_address: &String,call_button : &CallButton, id: &u32 ){
    println!("[Client] sending order:");

    let mut stream = TcpStream::connect(server_address).expect("could not connect");
    let order = tcp_message::set_order {
        order: TcpLiftOrderT {
            button_type: ButtonType::from_u8(call_button.call),
            floor: call_button.floor as u32 ,
            elevator_id: *id,
        },
    };
    let serialized_message = bincode::serialize(&order).unwrap();
    match stream.write(&serialized_message) {
        Ok(_) => {
            /*println!(
                "[client] ({:?}) {:?} sending: {:?}",
                i,
                stream.local_addr().unwrap(),
                order
            );*/
        }
        Err(_) => {

        }
    }
}