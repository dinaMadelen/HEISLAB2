use std::io::Read;
use std::io::Write;
use std::net::SocketAddr;

use driver_rust::elevio;
use serde;
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc;
use std::sync::{Arc, Mutex, RwLock};
use std::thread;
use std::time::Duration;

use tokio::sync::broadcast;
use std::collections::HashMap;

#[path = "../elevator_description/mod.rs"]
use crate::elevator_description;
//mod elevator_description;


use elevator::*;

#[cfg(feature = "silent")]
macro_rules! dbg {
    ($val:expr) => {
        ()
    }; // No-op for tests, suppress output
}



#[tokio::main]
async fn tcp_server(listening_address: &str) -> Arc<std::sync::Mutex<ElevatorDB>> {
    let listener = TcpListener::bind(listening_address).expect("failed binding");
    let start_time = std::time::SystemTime::now();
    //
    //let (tx,rx)= mpsc::channel();
    //let rx = Arc::new(Mutex::new(rx)); // is not clonable by default. tx is.
    let (tx, _) = broadcast::channel::<Vec<u8>>(4096);

    let mut DB = Arc::new(Mutex::new(ElevatorDB::new()));
    listener
        .set_nonblocking(true)
        .expect("error changing server settings");
    loop {
        // TODO: rewrite to instead terminate when no new connections AND no active connections.
        if start_time.elapsed().expect("error decifiring time") > Duration::from_secs(8) {
            dbg!(&DB);
            break; // terminate server after some time
        }
        match listener.accept() {
            Ok((stream, addr)) => {
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
                dbg!("fads");
            }
            Err(e) => {}
        }
    }

    return DB;
}

fn tcp_server_stream_handler(
    mut stream: TcpStream,
    address: SocketAddr,
    mut rx: tokio::sync::broadcast::Receiver<Vec<u8>>,
    tx: tokio::sync::broadcast::Sender<Vec<u8>>,
    mutex_DB: Arc<Mutex<ElevatorDB>>,
) {
    let mut buffer: [u8; 4096] = [0; 4096];
    let mut timeout = std::time::SystemTime::now(); // used for detecting missing client
    stream
        .set_read_timeout(Some(Duration::from_millis(1000)))
        .expect("error changing server settings");
    stream
        .set_nonblocking(true)
        .expect("error changing server settings");
    dbg!(&stream);
    loop {
        match stream.read(&mut buffer) {
            Ok(0) => {
                //dbg!("connection closed");
                {
                    //let DB = mutex_DB.lock().unwrap();
                    //DB.dbg();
                }
                break;
            }
            Ok(size) => {
                //let a = buffer.clone();
                //dbg!("buffer: {:?}" , String::from_utf8_lossy(&buffer[..size]));

                timeout = std::time::SystemTime::now();
                match bincode::deserialize::<tcp_message>(&buffer[..size]) {
                    Ok(tcp_message::set_order { order }) => {
                        let mut DB = mutex_DB.lock().unwrap();
                        let result = DB.allocate_to_best_id(order);
                        if result != None {
                            //someone got the order
                            // spread the message to the other lift connections.
                            let mut _order = order.clone();
                            _order.elevator_id = result.unwrap();
                            dbg!(_order);
                            let message: tcp_message = tcp_message::set_order { order: _order };
                            let serialized_message = bincode::serialize(&message).unwrap();
                            //dbg!("serialized: {:?}", serialized_message);
                            println!("[server] order{:?} : ", order);

                            tx.send(serialized_message);
                        }
                    }
                    Ok(tcp_message::clear_order { order }) => {
                        let mut DB = mutex_DB.lock().unwrap();

                        let result = DB.clear(order);

                        if result == Ok(()) {
                            //check that there actually was a order to clear.
                            // spread the message to the other lift connections.
                            //let mut _order = order.clone();
                            //_order.elevator_id=result.unwrap();
                            //dbg!("order::  {:?}",_order);
                            //let message: tcp_message = tcp_message::set_order{order : _order };

                            let message: tcp_message = tcp_message::clear_order { order: order };
                            let serialized_message = bincode::serialize(&message).unwrap();
                            println!("[server] tokio{:?} : {:?}", address, &message);
                            tx.send(serialized_message);
                        } else {
                            panic!("no such job {:?} \n {:?}", order, &DB);
                        }
                    }
                    Ok(tcp_message::NOP {
                        elevator_id: _message_id,
                    }) => {
                        // do noting. only for reseting timer.
                    }
                    _ => {
                        dbg!("could not interpret message");
                    }
                }
            }
            //let response = format!("{} said: {}", address.to_string(), recieved);
            //stream.write_all(response.as_bytes()).unwrap();
            Err(e) => {
                //dbg!("[server] lost connection to {:?}. reallocating",address);
            }
        }
        {
            match rx.try_recv() {
                Ok(message) => {
                    //dbg!("sending message {:?}",message);
                    match stream.write_all(&message) {
                        Ok(_) => {
                            timeout = std::time::SystemTime::now();
                        }
                        Err(e) => {
                            dbg!(address);
                            // TODO. implement realocate
                            break;
                        }
                    }
                    _ = stream.flush();
                    {
                    let message_u8 : &[u8] = &message;
                    let deserialized = bincode::deserialize::<tcp_message>(message_u8);
                    println!("[server]  sending ({:?}):  {:?}", address, deserialized );
                    }
                    dbg!(address);
                }
                Err(tokio::sync::broadcast::error::TryRecvError::Empty) => {}
                Err(tokio::sync::broadcast::error::TryRecvError::Closed) => {
                    break;
                }
                Err(tokio::sync::broadcast::error::TryRecvError::Lagged(_)) => {}
            };
        }
        if timeout.elapsed().expect("error decifiring time") > Duration::from_millis(1000) {
            dbg!(address);
            // TODO: implement realocation.

            break;
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
struct ElevatorDB {
    orders_as_id: [[u32; N_BUTTONS]; N_FLOORS], // store order id
}

impl ElevatorDB {
    pub fn new() -> Self {
        ElevatorDB {
            orders_as_id: [[0; N_BUTTONS]; N_FLOORS],
        }
    }
    // give the order to the id which fits best. return None if a lift is handeling the order
    pub fn allocate_to_best_id(&mut self, lift_order: TCP_lift_order_t) -> Option<u32> {
        //dbg!("button: {:?} ({:?}) floor: {:?} ({:?})", lift_order.button_type.to_u8(),N_BUTTONS ,lift_order.floor,N_FLOORS);
        if ((self.orders_as_id[lift_order.floor as usize][lift_order.button_type.to_u8() as usize])
            != 0)
        {
            return None; // the order is already handled by a elevator.
        }
        let result = self.find_least_orders(lift_order.elevator_id);
        let mut _order = lift_order.clone();
        _order.elevator_id = result.unwrap();
        self.set(_order);
        return result;
    }
    // get all relevant orders of specified id
    pub fn get_by_id(&self, id: u32) -> [[bool; N_BUTTONS]; N_FLOORS] {
        let mut output: [[bool; N_BUTTONS]; N_FLOORS] = [[false; N_BUTTONS]; N_FLOORS];
        for (row_index, row) in self.orders_as_id.iter().enumerate() {
            for (col_index, &value) in row.iter().enumerate() {
                if value == id {
                    output[row_index][col_index] = true;
                }
            }
        }
        return output;
    }
    // force index to match order
    pub fn set(&mut self, lift_order: TCP_lift_order_t) -> Result<(), u32> {
        if lift_order.elevator_id != 0 {
            self.orders_as_id[lift_order.floor as usize][lift_order.button_type.to_u8() as usize] =
                lift_order.elevator_id;
            return Ok(());
        }
        return Err(1);
    }
    pub fn clear(&mut self, lift_order: TCP_lift_order_t) -> Result<(), u32> {
        if (self.orders_as_id[lift_order.button_type.to_u8() as usize][lift_order.floor as usize]
            == 0)
        {
            return Err(1);
        }
        self.orders_as_id[lift_order.button_type.to_u8() as usize][lift_order.floor as usize] = 0;
        Ok(())
    }

    pub fn print(&self) {
        dbg!(&self.orders_as_id); // dbg the database
    }

    pub fn remove_and_recalculate(&mut self, id: u32) {
        let mut output: [[bool; N_BUTTONS]; N_FLOORS] = [[false; N_BUTTONS]; N_FLOORS];
        let _DB = self.clone();
        for (row_index, row) in _DB.orders_as_id.iter().enumerate() {
            for (col_index, &value) in row.iter().enumerate() {
                if value == id {
                    self.orders_as_id[row_index as usize][col_index as usize] = 0;
                    output[row_index][col_index] = true;
                }
            }
        }
    }

    // this implementation can give problems if a "ghost" elevator appears
    pub fn find_least_orders(&self, id: u32) -> Option<u32> {
        let mut counts = HashMap::new();
        // insert the id of the sender, since it might be missing.
        *counts.entry(id).or_insert(0);
        // Iterate over each row in the 2D array
        for row in self.orders_as_id.iter() {
            for &num in row.iter() {
                if num != 0 {
                    // Count occurrences of each number, ignoring 0
                    *counts.entry(num).or_insert(0) += 1;
                }
            }
        }

        // Find the number with the fewest occurrences
        counts
            .into_iter()
            .min_by_key(|&(_, count)| count)
            .map(|(num, _)| num)
    }
    // removes all occurences of a id and replaces it with another.
}

#[cfg(test)]
mod tests {

    use super::*;
    #[test]
    fn smoketest_tcp_backup() {
        // start up some servers and clients localy on the machine.
        //check that all clients and servers are syncronized at the end of some communication.
        // it fails when there are too many clients (lost packets).
        let connecting_address = "127.0.0.1:3423";

        let server_handle = thread::spawn(move || tcp_server(connecting_address));

        /*let handles: Vec<_> = (1..2)
            .map(|id| thread::spawn(move || dummy_tcp_client(connecting_address, id)))
            .collect();
        */
        let client_handles= thread::spawn(move || dummy_tcp_client(connecting_address, 1));
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
        println!("client started {:?}",id);
        stream
            .set_nonblocking(true)
            .expect("error changing server settings");
        stream.set_read_timeout(Some(Duration::from_millis(10)));

        let mut buffer = [0; 4096];

        let mut wait_to_send_timer = std::time::SystemTime::now();
        let mut i = id;
        let mut timeout = std::time::SystemTime::now();
        while (i <= (id + 20)) {
            //thread::sleep(Duration::from_millis(100));
            if wait_to_send_timer.elapsed().expect("error decifiring time")
                > Duration::from_millis(500 + id as u64)
            {
                if ((i <= id + 10) && ((i % 3) != 2)) {
                    //println!("fasdf{:?}", i);
                    let order = tcp_message::set_order {
                        order: TCP_lift_order_t {
                            button_type: ButtonType::from_u8(i as u8 % N_BUTTONS as u8),
                            floor: ((i as u32 % N_FLOORS as u32) as u32),
                            elevator_id: id,
                        },
                    };
                    let serialized_message = bincode::serialize(&order).unwrap();
                    match stream.write_all(&serialized_message) {
                        Ok(_) => {
                            println!(
                                "[client] ({:?}) {:?} sending: {:?}",
                                i,stream.local_addr().unwrap(),
                                order
                            );
                            timeout = std::time::SystemTime::now();
                        }
                        Err(_) => {
                            break;
                        }
                    }
                }
                if (i <= id + 10) && ((i % 3) == 2) {
                    let orders_to_do = DB.get_by_id(id);
                    match (find_first_true(&orders_to_do)) {
                        Some((button_type, floor)) => {
                            let order = tcp_message::clear_order {
                                order: TCP_lift_order_t {
                                    button_type: ButtonType::from_u8(button_type as u8),
                                    floor: ((floor as u8) as u32),
                                    elevator_id: id,
                                },
                            };
                            let serialized_message = bincode::serialize(&order).unwrap();

                            match stream.write_all(&serialized_message) {
                                Ok(_) => {
                                    println!(
                                        "[client] {:?} sending: {:?}",
                                        stream.local_addr().unwrap(),
                                        order
                                    );
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

            match stream.read(&mut buffer) {
                Ok(0) => {
                    dbg!("connection closed");
                    break;
                }
                Ok(size) => {
                    timeout = std::time::SystemTime::now();
                    //let a = buffer.clone();
                    //dbg!("buffer: {:?}" , String::from_utf8_lossy(&buffer[..size]));

                    match bincode::deserialize::<tcp_message>(&buffer[..size]) {
                        Ok(tcp_message::set_order { order }) => {
                            _ = DB.set(order);
                            if (id == 1) {
                                dbg!(order);
                            }
                            //dbg!("new order to: {:?}", id);
                            //let result= DB.allocate_to_best_id(order);
                        }
                        Ok(tcp_message::clear_order { order }) => {
                            let result = DB.clear(order);
                            println!("[client] order cleared ({:?}) : {:?}", result, order);
                        }
                        Ok(tcp_message::NOP { elevator_id }) => {}
                        _ => {
                            //dbg!("could not interpret message");
                        }
                    }
                }
                Err(e) =>{
                    //dbg!(e);
                }
            }
        }

        println!("{:?}",&DB);
        dbg!(id);
        return DB;
    }

    fn find_first_true(arr: &[[bool; N_BUTTONS]; N_FLOORS]) -> Option<(usize, usize)> {
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
