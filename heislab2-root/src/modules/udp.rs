

#![allow(warnings)]
//UDP Functions for sending and reciving data over UDP

/*----------------------Left to IMPLEMENT:

Threading

------------------------Structs in this file:

UdpMsg                Contains message data and overhead
UdpHeader             Contains overhead

-----------------------------------------------Functions in this file:
serialize              UdpMsg -> Vec<u8>
deserialize            Vec<u8> -> UdpMsg
calc_checksum          Calculate checksum to a u8
comp_checksum          Compare a recived UdpMsg checksum with the calculated checksum
udp_send               Ensured message integrity
udp_recive             returns UdpMsg struct
udp_broadcast          Not ensured message integrity
udp_recive_ensure      same as recive but verfiies that the message is correct
udp_send_ensure        same as send, but requrires ACK

------------------------------------------------Message IDs

0:  Master Wordview (Master Broadcasts worldview hash)
1:  Ack (Data variable contains massage ID of the message it is responding to)
2:  Nak (Data variable contains massage ID of the message it is responding to)
3:  New Master (Master Broadcasts)
4:  New Online (Slave Broadcast)
5:  Request Queue (Data contains ID of queue that)
6:  Response to Queue (Master sends queue)
7:  Error: Exisiting worldview hash does not match Slave's worldview hash (Slave sends)
8:  Error: Can't service queue/ Going Offline (Master/Slave)
10: Error: Any

----------------------------------------------- !!!OBS!!! ADD TO Cargo.toml:

[dependencies]
serde = { version = "1", features = ["derive"] }
bincode = "1"
sha2 = { version = "0.11.0-pre.4" }

*/


//----------------------------------------------Imports
use std::net::{SocketAddr, UdpSocket}; // https://doc.rust-lang.org/std/net/struct.UdpSocket.html
                                       //use std::sync::{Arc, Mutex};          // https://doc.rust-lang.org/std/sync/struct.Mutex.html
use serde::{Deserialize, Serialize}; // https://serde.rs/impl-serialize.html         //Add to Cargo.toml file, Check comment above
                                     // https://docs.rs/serde/latest/serde/ser/trait.Serialize.html#tymethod.serialize
use bincode; use sha2::digest::Update;
// https://docs.rs/bincode/latest/bincode/      //Add to Cargo.toml file, Check comment above
use sha2::{Digest, Sha256}; // https://docs.rs/sha2/latest/sha2/            //Add to Cargo.toml file, Check comment above
use std::time::Duration; // https://doc.rust-lang.org/std/time/struct.Duration.html
use std::thread::sleep; // https://doc.rust-lang.org/std/thread/fn.sleep.html
use crate::modules::elevator;
use crate::modules::slave;
use crate::modules::master;

//----------------------------------------------Enum
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum message_type {

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

//----------------------------------------------Structs

#[derive(Debug, Serialize, Deserialize, Clone)] // this is needed to serialize message
//UDP Header
pub struct UdpHeader {
    sender_id: u8,            // ID of the sender of the message.
    message_id: message_type, // ID for what kind of message it is, e.g. Button press, or Update queue.
    checksum: Vec<u8>,        // Hash of data to check message integrity.
}

#[derive(Debug, Serialize, Deserialize, Clone)] // this is needed to serialize message
                                                //UDP Message Struct
pub struct UdpMsg {
    header: UdpHeader, // Header struct containing information about the message itself
    data: Vec<u8>,     // Data so be sent.
}

//----------------------------------------------Functions

fn make_Udp_msg(elevator: crate::modules::elevator::Elevator, message_type: message_type, message: Vec<u8>) -> UdpMsg {
    let hash = calc_checksum(&message);
    let mut overhead = UdpHeader {
        sender_id: elevator.ID,
        message_id: message_type,
        checksum: hash,
    };
    let msg = UdpMsg {
        header: overhead,
        data: message,
    };
    return msg;
}


//Recive UDP message
fn udp_recive(socket: &UdpSocket, max_wait: u8) -> Option<UdpMsg> {
    socket
        .set_read_timeout(Some(Duration::new(max_wait.into(), 0)))
        .expect(&format!("Failed to set read timeout of {}s", max_wait));

    let mut buffer = [0; 1024];

    //Recive message
    let msg = match socket.recv_from(&mut buffer){
        Ok((size, sender)) => {
            println!("Message size {}, from {}", size, sender);
            deserialize(&buffer[..size]);
        }
        Err(e) => {
            println!("Failure to recive:{}", e);
            return None;
        }
    };

    //Categorize message
    println!("Messagetype:{}",msg.message_id);

    match msg.message_id{

        message_type::Wordview=>{
            update_from_worldview(&slave, msg.data);
            handle_multiple_masters(me: &Elevator, sender: &Elevator, worldview: &Worldview);
        }

        message_type::Ack=>{
            // Recive ACK
            print!("👍")
        }

        message_type::Nak=>{
            // Recive NAK
            print!("👎")            
        }

        message_type::New_Order=>{
            receive_order(&mut slave, msg.data);
        }

        message_type::New_Master=>{
            //set new master
        }

        message_type::New_Online=>{
            //add to active elevators
        }
            
        message_type::Respond_Queue=>{
            //send queues
        }

        message_type::Error_Worldview=>{
            notify_wordview_error(msg.slave_id: u8, msg.data);
        }

        message_type::Error_Offline=>{
            //set_offline
            msg.slave_id
            reassign_orders(orders:Vec<u8>);
            //remove from active elevators
        }

        message_type::Request_Resend=>{
         // Does this anyway if it does not respond?   
        }

        _=>{
            println!("Unreadable message recived")
        }

        }

}


// Split UdpMsg into bytes
fn serialize(msg: &UdpMsg) -> Vec<u8> {
    let serialized_msg = bincode::serialize(msg).expect("Failed to serialize message");
    return serialized_msg;
}

// Combine bytes in message buffer into UdpMsg
fn deserialize(buffer: &[u8]) -> Option<UdpMsg> {
    let deserialized_msg = bincode::deserialize(buffer).ok();
    return deserialized_msg;
}

// Calculate Checksum.
fn calc_checksum(data: &Vec<u8>) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let hash = hasher.finalize();
    return hash.to_vec();
}

// Compare checksums, Not sure if we need this or not
fn comp_checksum(msg: &UdpMsg) -> bool {
    return calc_checksum(&msg.data) == msg.header.checksum;
}


//ACK
fn udp_ack(socket: &UdpSocket, target_address: SocketAddr) -> bool {
    let thumbs_up = "👍".as_bytes();
    match socket.send_to(thumbs_up, target_address) {
        Ok(_) => {
            println!("Sendt ACK");
            return true;
        }
        Err(e) => {
            println!("Error sending ACK: {}", e);
            return false;
        }
    }
}

//NAK
fn udp_nak(socket: &UdpSocket, target_address: SocketAddr) -> bool {
    let thumbs_down = "👎".as_bytes();
    match socket.send_to(thumbds_down, target_address) {
        Ok(_) => {
            println!("Sendt NAK");
            return true;
        }
        Err(e) => {
            eprintln!("Error sending NAK: {}", e);
            return false;
        }
    }
}

//Sending UDP message
fn udp_send(socket: &UdpSocket, target_adress: SocketAddr, msg: &UdpMsg) -> bool {
    let data = serialize(msg);
    match socket.send_to(&data, target_adress) {
        Ok(_) => {
            println!("Message sent to: {}", target_adress);
            return true;
        }
        Err(e) => {
            eprintln!("Error sending message: {}", e);
            return false;
        }
    }
}

//Broadcast
fn udp_broadcast(msg: &UdpMsg) {
    let socket = UdpSocket::bind("0.0.0.0:0").expect("Failed to bind socket, broadcast");
    socket
        .set_broadcast(true)
        .expect("failed to activate broadcast");

    let msg = serialize(msg);
    let target_address = "255.255.255.255;20000";

    match socket.send_to(&msg, target_address) {
        Ok(_) => {
            println!("Broadcast successful");
        }
        Err(e) => {
            eprintln!("Error sending data: {}", e);
        }
    }
}

// Sending UDP, with retry
fn udp_send_ensure(socket: &UdpSocket, target_addr: &str, msg: &UdpMsg, max_retry: u8) -> bool {
    let data = serialize(msg);
    let mut retries = max_retry;

    while retries > 0 {
        match socket.send_to(&data, target_addr) {
            Ok(_) => {
                println!("Message sent to: {}", target_addr);
            }
            Err(e) => {
                eprintln!("Error sending message: {}", e);
            }
        }

        match socket.send_to(&data, target_addr) {
            Ok(_) => {
                println!("Sent message {}", msg.header.sequence_number);
            }
            Err(e) => {
                eprintln!("Send error: {}", e);
                retries -= 1;
                continue;
            }
        }

        let mut buffer = [0; 1024];
        match socket.recv_from(&mut buffer) {
            Ok((_, rec_addr)) if rec_addr.to_string() == target_addr => {
                // Any empty or accepted message
                let thumbs_up = "👍".as_bytes();
                if buffer[0] == thumbds_up {
                    // ACK received, ASCII for ACK
                    println!("ACK received for {}", msg.header.sequence_number);
                    // Message sucessfully sent and recived
                    return true; 
                }
            }
            _ => retries -= 1, // Anything other than an empty or accepted message
        }
    }

    println!("Failed to send after retries.");
    return false;
}

// Reciving UDP, with ACK
fn udp_receive_ensure(socket: &UdpSocket, max_wait: u8) -> Option<UdpMsg> {
    socket
        .set_read_timeout(Some(Duration::new(max_wait.into(), 0)))
        .expect("Failed to set read timeout");
    let mut buffer = [0; 1024];

    match socket.recv_from(&mut buffer) {
        Ok((size, sender_addr)) => {
            if let Some(msg) = deserialize(&buffer[..size]) {
                if calc_checksum(&msg.data) == msg.header.checksum {
                    udp_ack(socket, sender_addr); // Send ACK
                    return Some(msg);
                } else {
                    udp_nak(socket, sender_addr); // Send NAK
                }
            }
        }
        Err(e) => {
            eprintln!("error reciving{}", e);
        }
    }
    return None;
}

//------------------------------Tests-----------------------

#[cfg(test)] // https://doc.rust-lang.orgbook/ch11-03-test-organization.html Run tests with "cargo test"
mod tests {
    use super::*;
    use std::net::UdpSocket;

    #[test]
    fn test_serialize_deserialize() {
        let msg = UdpMsg {
            header: UdpHeader {
                sender_id: 1,
                message_id: message_type::Ack,
                checksum: vec![0x12, 0x34],
            },
            data: vec![1, 2, 3, 4],
        };

        let serialized = serialize(&msg);
        let deserialized = deserialize(&serialized).expect("Deserialization failed");

        assert_eq!(msg.header.sender_id, deserialized.header.sender_id);
        assert_eq!(
            msg.header.sequence_number,
            deserialized.header.sequence_number
        );
        assert_eq!(msg.data, deserialized.data);
    }

    #[test]
    fn test_calc_checksum() {
        let data = vec![1, 2, 3, 4];
        let checksum = calc_checksum(&data);
        assert!(!checksum.is_empty());
    }

    #[test]
    fn test_comp_checksum() {
        let data = vec![1, 2, 3, 4];
        let checksum = calc_checksum(&data);
        let msg = UdpMsg {
            header: UdpHeader {
                sender_id: 1,
                message_id: message_type::Ack,
                checksum,
            },
            data,
        };
        assert!(comp_checksum(&msg));
    }

    #[test]
    fn test_udp_send_recv() {
        // sleep(Duration::from_millis(1000));
        let socket = UdpSocket::bind("127.0.0.1:0").expect("Failed to bind socket");
        let local_addr = socket.local_addr().expect("Failed to get socket address");

        let msg = UdpMsg {
            header: UdpHeader {
                sender_id: 1,
                message_id: message_type::Ack,
                checksum: vec![0x12, 0x34],
            },
            data: vec![1, 2, 3, 4],
        };

        let send_socket = socket.try_clone().expect("Failed to clone socket");
        let recv_socket = socket;

        let msg_clone = msg.clone();
        std::thread::spawn(move || {
            //delay?
            udp_send(&send_socket, local_addr, &msg_clone);
        });

        let received_msg = udp_recieive(&recv_socket, 5).expect("Failed to receive message");
        assert_eq!(msg.data, received_msg.data);
    }
}