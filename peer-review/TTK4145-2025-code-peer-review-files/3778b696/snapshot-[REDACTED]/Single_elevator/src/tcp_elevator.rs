use std::io::Read;
use std::io::Write;
use std::net::SocketAddr;


use serde;
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use tokio::sync::broadcast;

use std::collections::HashMap;

use crate::elevator::{N_FLOORS,ButtonType};
use crate::tcp_definitions::{ElevatorDB,TcpLiftOrderT,tcp_message};
//use crate::elevator::elevator::N_SHARED_BUTTONS
#[path = "elevator.rs"]
pub mod elevator;
use elevator::*;


#[cfg(feature = "silent")]
macro_rules! dbg {
    ($val:expr) => {
        ()
    }; // No-op for tests, suppress output
}



#[tokio::main]
pub async fn tcp_server(listening_address: String) -> Arc<std::sync::Mutex<ElevatorDB>> {
    let listener = TcpListener::bind(listening_address).expect("failed binding");
    let start_time = std::time::SystemTime::now();
    println!("[Server] bound to: {:?}",&listener);
    let a = elevator::N_SHARED_BUTTONS;
    //let (tx,rx)= mpsc::channel();
    //let rx = Arc::new(Mutex::new(rx)); // is not clonable by default. tx is.
    let (tx, _) = broadcast::channel::<Vec<u8>>(4096);

    let DB = Arc::new(Mutex::new(ElevatorDB::new()));
    listener
        .set_nonblocking(true)
        .expect("error changing server settings");
    loop {
        // TODO: rewrite to instead terminate when no new connections AND no active connections.
        if start_time.elapsed().expect("error decifiring time") > Duration::from_secs(8) {
            ////dbg!(&DB);
            //break; // terminate server after some time
        }
        match listener.accept() {
            Ok((stream, addr)) => {
                println!("[Server] new connection: {:?} {:?}", &stream, &addr);
                let handler_rx = tx.subscribe();
                let handler_tx = tx.clone();
                let handler_DB_ref = Arc::clone(&DB);
                thread::spawn(move || {
                    tcp_server_stream_handler(stream, addr, handler_rx, handler_tx, handler_DB_ref)
                });
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                // no new connection
                thread::sleep(Duration::from_millis(400));
            }
            Err(e) => {}
        }
        //println!("{:?}",DB);
    }

    return DB;
}

fn tcp_server_stream_handler(
    mut stream: TcpStream,
    address: SocketAddr,
    mut rx: tokio::sync::broadcast::Receiver<Vec<u8>>,
    tx: tokio::sync::broadcast::Sender<Vec<u8>>,
    mutex_db: Arc<Mutex<ElevatorDB>>,
) {
    let mut buffer: [u8; 4096] = [0; 4096];
    let mut timeout = std::time::SystemTime::now(); // used for detecting missing client
    stream
        .set_read_timeout(Some(Duration::from_millis(1000)))
        .expect("error changing server timeout");
    stream
        .set_nonblocking(true)
        .expect("error changing server to none blocking");
    //dbg!(&stream);
    loop {
        match stream.read(&mut buffer) {
            Ok(0) => {
                ////dbg!("connection closed");
                {
                    //let DB = mutex_DB.lock().unwrap();
                    //DB.dbg();
                }
                break;
            }
            Ok(size) => {

                timeout = std::time::SystemTime::now();
                // TODO: thhis method may give errors if more than one message is recieved at once.
                match bincode::deserialize::<tcp_message>(&buffer[..size]) {
                    Ok(tcp_message::set_order { order }) => {
                        let mut db = mutex_db.lock().unwrap();
                        let result  : Option<TcpLiftOrderT> = db.allocate_to_best_id(order);
                        if result != None {
                            //someone got the order
                            // spread the message to the other lift connections.
                            let message : tcp_message = tcp_message::set_order { order: result.unwrap() };
                            let serialized_message = bincode::serialize(&message).unwrap();
                            ////dbg!("serialized: {:?}", serialized_message);
                            //println!("[server] tokio: {:?} : ", result);

                            _= tx.send(serialized_message);
                        }
                    }
                    Ok(tcp_message::clear_order { order }) => {
                        let mut DB = mutex_db.lock().unwrap();

                        let result = DB.clear(order);

                        if result == Ok(()) {
                            //check that there actually was a order to clear.
                            // spread the message to the other lift connections.

                            let message: tcp_message = tcp_message::clear_order { order: order };
                            let serialized_message = bincode::serialize(&message).unwrap();
                            //println!("[server] tokio: {:?} : {:?}", address, &message);
                            _=tx.send(serialized_message);
                        } else {
                            // TODO: the server which sent the clear order should get a clear signal.
                            
                            
                            //panic!("no such job {:?} \n {:?}", order, &DB);
                        }
                    }
                    Ok(tcp_message::NOP {
                        elevator_id: _message_id,
                    }) => {
                        // do noting. only for reseting timer.
                    }
                    _ => {
                        //dbg!("could not interpret message");
                    }
                }
            }
            Err(e) => {
                //TODO: implement realocate.
                ////dbg!("[server] lost connection to {:?}. reallocating",address);
            }
        }
        {
            match rx.try_recv() {
                Ok(message) => {
                    ////dbg!("sending message {:?}",message);
                    match stream.write_all(&message) {
                        Ok(_) => {
                            timeout = std::time::SystemTime::now();
                        }
                        Err(e) => {
                            //dbg!(address);
                            //println!("[server] could not send");
                            // TODO. implement realocate
                            break;
                        }
                    }
                    _ = stream.flush();
                    {
                        let message_u8: &[u8] = &message;
                        let deserialized = bincode::deserialize::<tcp_message>(message_u8);
                        //println!("[server]  sending ({:?}):  {:?}", message, deserialized);
                    }
                    //dbg!(address);
                }
                Err(tokio::sync::broadcast::error::TryRecvError::Empty) => {}
                Err(tokio::sync::broadcast::error::TryRecvError::Closed) => {
                    break;
                }
                Err(tokio::sync::broadcast::error::TryRecvError::Lagged(_)) => {}
            };
        }
        if timeout.elapsed().expect("error decifiring time") > Duration::from_millis(1000) {
            //dbg!(address);
            // TODO: implement realocation.

            //break;
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CabButtonsT{
    button_state : [bool ; elevator::N_FLOORS],
    id : u32,
}


#[cfg(test)]
mod tests {

    use super::*;
    #[test]
    fn smoketest_tcp_backup() {
        // start up some servers and clients localy on the machine.
        //check that all clients and servers are syncronized at the end of some communication.
        // it fails when there are too many clients (lost packets).
        let connecting_address: String = "127.0.0.1:3423".to_string();

        let server_handle = thread::spawn(move || tcp_server(connecting_address));

        /*let handles: Vec<_> = (1..2)
            .map(|id| thread::spawn(move || dummy_tcp_client(connecting_address, id)))
            .collect();
        */
        let client_handles = thread::spawn(move || dummy_tcp_client(connecting_address, 1));
        let serverDB = server_handle.join().unwrap();

        //for handle in handles {
        let clientDB = client_handles.join().unwrap();
        {
            let mut serverDB = serverDB.lock().unwrap();
            assert_eq!(*serverDB, clientDB);
        }
        //}
    }

    fn dummy_tcp_client(connection_address: &str, id: u32) -> ElevatorDB {
        thread::sleep(Duration::from_millis(1000));
        let mut stream = TcpStream::connect(connection_address).expect("could not connect");
        let mut DB: ElevatorDB = ElevatorDB::new();
        // send "message number<i>" and wait on answer. dbg answer to terminal.
        thread::sleep(Duration::from_millis(200));
        //println!("client started {:?}", id);
        stream
            .set_nonblocking(true)
            .expect("error changing server settings");
        stream.set_read_timeout(Some(Duration::from_millis(10)));

        let mut tcp_buffer = [0; 4096];
        let mut wait_to_send_timer = std::time::SystemTime::now();
        let mut i = id;
        let mut timeout = std::time::SystemTime::now();
        while (i <= (id + 20)) {
            //thread::sleep(Duration::from_millis(100));
            if wait_to_send_timer.elapsed().expect("error decifiring time")
                > Duration::from_millis(500 + id as u64)
            {
                if ((i <= id + 10) && ((i % 3) != 2)) {
                    ////println!("fasdf{:?}", i);
                    let order = tcp_message::set_order {
                        order: TcpLiftOrderT {
                            button_type: ButtonType::from_u8(
                                i as u8 % N_BUTTONS as u8,
                            ),
                            floor: ((i as u32 % elevator::N_FLOORS as u32) as u32),
                            elevator_id: id,
                        },
                    };
                    let serialized_message = bincode::serialize(&order).unwrap();
                    match stream.write_all(&serialized_message) {
                        Ok(_) => {
                            /*println!(
                                "[client] ({:?}) {:?} sending: {:?}",
                                i,
                                stream.local_addr().unwrap(),
                                order
                                
                            );*/
                            timeout = std::time::SystemTime::now();
                        }
                        Err(_) => {
                            break;
                        }
                    }
                }
                if (i <= id + 10) && ((i % 3) == 2) {
                    let mut orders_to_do = DB.get_all_buttons_as_vec(id);
                    match orders_to_do.pop() {
                        Some(order) =>{

                            let order= tcp_message::clear_order { order: order };
                            let serialized_message = bincode::serialize(&order).unwrap();

                            match stream.write_all(&serialized_message) {
                                Ok(_) => {
                                    /*println!(
                                        "[client] {:?} sending: {:?}",
                                        stream.local_addr().unwrap(),
                                        order
                                    ); */
                                    timeout = std::time::SystemTime::now();
                                }
                                Err(_) => {
                                    break;
                                }
                            }
                        }
                        None => {}
                    }
                    
                }
                if (i > id + 10) {
                    let order = tcp_message::NOP { elevator_id: id };
                    let serialized_message = bincode::serialize(&order).unwrap();
                    match stream.write_all(&serialized_message) {
                        Ok(_) => {
                            timeout = std::time::SystemTime::now();
                        }
                        Err(_) => {
                            break;
                        }
                    }
                }

                stream.flush();

                wait_to_send_timer = std::time::SystemTime::now();
                i += 1;
            }

            match stream.read(&mut tcp_buffer) {
                Ok(0) => {
                    //dbg!("connection closed");
                    break;
                }
                Ok(size) => {
                    timeout = std::time::SystemTime::now();
                    //let a = buffer.clone();
                    ////dbg!("buffer: {:?}" , String::from_utf8_lossy(&buffer[..size]));

                    match bincode::deserialize::<tcp_message>(&tcp_buffer[..size]) {
                        Ok(tcp_message::set_order { order }) => {
                            _ = DB.set(order);
                            if (id == 1) {
                                //dbg!(order);
                            }
                            ////dbg!("new order to: {:?}", id);
                            //let result= DB.allocate_to_best_id(order);
                        }
                        Ok(tcp_message::clear_order { order }) => {
                            let result = DB.clear(order);
                            //println!("[client] order cleared ({:?}) : {:?}", result, order);
                        }
                        Ok(tcp_message::NOP { elevator_id }) => {}
                        _ => {
                            ////dbg!("could not interpret message");
                        }
                    }
                }
                Err(e) => {
                    ////dbg!(e);
                }
            }
        }

        //println!("{:?}", &DB);
        //dbg!(id);
        return DB;
    }

    fn find_first_true(
        arr: &[[bool; elevator::N_BUTTONS]; elevator::N_FLOORS],
    ) -> Option<(usize, usize)> {
        for (i, row) in arr.iter().enumerate() {
            for (j, &val) in row.iter().enumerate() {
                if val {
                    return Some((i, j)); // Return the index of the first `true` value
                }
            }
        }
        None // Return None if there are no `true` values in the array
    }
}
