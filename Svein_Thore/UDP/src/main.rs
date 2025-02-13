//UDP Functions for sending and reciving data over UDP

/*----------------------Left to IMPLEMENT:
Sequence number for correct order, Threading ,Mutex

------------------------Structs in this file
udp_msg                Contains message data and overhead
-----------------------------------------------Functions in this file
serilize               udp_msg -> Vec<u8>
deserialize            Vec<u8> -> udp_msg
x- calc_checksum          Calculate checksum to a u8
x- comp_checksum          Compare a recived udp_msg checksum with the calculated checksum
udp_send               Ensured message integrity
udp_recive             returns udp_msg struct
udp_broadcast          Not ensured message integrity

------------------------------------------------Message IDs

0:  Master Wordview (Master Broadcasts worldview hash)
1:  Ack (Data variable contains massage ID of the message it is responding to)
2:  Nak (Data variable contains massage ID of the message it is responding to)
3:  New Master (Master Broadcasts)
4:  New Online (Slave Broadcast)
5:  Request Queue (Data contains ID of queue that)
6:  Response to Queue (Master sends queue)
7:  Error: Exisiting worldview hash does not match Slave's worldview hash (Slave sends)
8:  Response to  Error: Worldview hash (Master responds with queues)
9:  Error: Can't service queue (Master/Slave)
10: Error: 

----------------------------------------------- !!!OBS!!! ADD TO Cargo.toml: 

[dependencies]
serde = "1"
bincode = "1"
----------------------------------------------- Example code
//Example Assigning mutex
let res_mutex = Arc::new(Mutex::new(0));            //Create a mutex
let res_mutex_recv = Arc::clone(&res_mutex);        //Clone a mutex from res_mutex
let res_mutex_send = Arc::clone(&res_mutex);        //Clone another mutex from res_mutex

//Exampel for adress  (IPv4:PORT)                   
let target_address = "127.0.0.1:20000";
let recive_address = "127.0.0.1:20001";             

//Example for socket (clone socket for multiple threads)
let socket = UdpSocket::bind("127.0.0.1:20001").expect("couldn't bind to address"); // Listening to
let socket_clone = socket.try_clone().expect("Failed to clone the socket");

*/

//----------------------------------------------Imports

use std::net::{UdpSocket, SocketAddr};  // https://doc.rust-lang.org/std/net/struct.UdpSocket.html
use std::sync::{Arc, Mutex};            // https://doc.rust-lang.org/std/sync/struct.Mutex.html
use serde::{Serialize, Deserialize};    // https://serde.rs/impl-serialize.html & https://docs.rs/serde/latest/serde/ser/trait.Serialize.html#tymethod.serialize
use bincode;                            // https://docs.rs/bincode/latest/bincode/      //Add to Cargo.toml file, Check comment above
use sha2::{Sha256, Digest};             // https://docs.rs/sha2/latest/sha2/            //Add to Cargo.toml file, Check comment above

//----------------------------------------------Struct

//UDP Header
struct udp_header{
    sender_id: u8,      // ID of the sender of the message.
    message_id: u8,     // ID for what kind of message it is, e.g. Button press, or Update queue.
    sequence_numb: u32, // Number of message in order.
    checksum: Vec<u8>,  // Hash of data to check message integrity.
}

//UDP Message Struct
struct udp_msg{
    header: udp_header, // Header struct containing information about the message itself
    data: Vec<u8>,      // Data so be sent.
}

//----------------------------------------------Functions

// Split udp_msg into bytes
fn serialize(msg: &udp_msg) -> Vec<u8> {

    let serialized_msg = bincode::serialize(msg).expect("Failed to serialize message");
    return serialized_msg;

}

// Combine bytes into udp_msg 
fn deserialize(buffer: &[u8]) -> Option<udp_msg> {

    let deserialized_msg = bincode::deserialize(buffer).ok(); 
    return deserialized_msg;
}

// Calculate Checksum. 
fn calc_checksum(data: &Vec<u8>) -> Str{

    let mut hasher sha256::new();
    hasher.update(data);
    let hash hasher.finalize();
    return hash; 
}

// Compare checksums, Not sure if i need this or not
fn comp_checksum(udp_msg: &udp_msg)-> bool{

    if (calc_checksum(&msg.data) == &msg.header.checksum){
        return true;
    }
    else{
        return false;
    }
}

//Recive UDP message
fn udp_recive(socket: &UdpSocket) -> Option<udp_msg>{

    let mut buffer = [0;1024];
    //let _lock = res_mutex_recv.lock().unwrap();
    //Recive message
    match socket.recv_from(&mut buffer){
 
        Ok(_) => {
            let msg = buffer[0];
            println!("Message size {}, from {}",size,sender);
            return Some(deserialize(&buffer[..size])?);
        }

        Err(e) => {
            println!("Failure to recive:{}",e);
            return None;
        }
    }
}

//ACK
fn udp_ack(target_address){

    match socket.send_to(0x06 , target_address) {
        Ok(_) => {
            println!("Sendt ACK");
        }
        Err(e) => {
            println!("Error sending ACK: {}", e);
        }
    }
}

//NAK
fn udp_nak(target_address){

    match socket.send_to(0x15 , target_address) {
        Ok(_) => {
            println!("Sendt NAK");
        }
        Err(e) => {
            eprintln!("Error sending NAK: {}", e);
        }
    }
}    

//Sending UDP message
fn udp_send(socket: &UdpSocket,target_adress: &str,msg: &udp_msg){

    let data = serialize(msg);
    match socket.send_to(&data,target){
        Ok(_) => {
            println!("Message sent to: {}",target";
        }
        Err(e) => {
            eprintln!("{}",e);
        }
    }
}

//Broadcast
fn udp_broadcast(data){

    let socket = UdpSocket::bind("0.0.0.0:0").expect("Failed to bind socket, broadcast");
    socket.set_broadcast(true).expect("failed to activate broadcast");

    let msg = serialize(data);

    match socket.send_to(&msg,"255.255.255.255;20000") {
        Ok(_) => {
            println!("Broadcast successful");
        }
        Err(e) => {
            eprintln!("Error sending data: {}", e);
        }
    }
}

// Sending UDP, with retry
fn udp_send_ensure(socket: &UdpSocket, target_addr: &str, msg: &ReliableUdpMsg) -> bool {

    let data = serialize(msg);
    let mut retries = 3;

    while retries > 0 {

        let data = serialize(msg);
        match socket.send_to(&data,target_addr){
            Ok(_) => {
                println!("Message sent to: {}",target_addr);
            }
            Err(e) => {
                eprintln!("{}",e);
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
            Ok((_,rec_addr)) if rec_addr.to_string() == target_addr => { // Any empty or accepted message
                if buffer[0] == 0x06 { // ACK received
                    println!("ACK received for {}", msg.header.sequence_number);
                    return true;
                }
            }
            .. => retries -= 1,  // Anything other than an empty or accepted message
        }
    }

    println!("Failed to send after retries.");
    return false; 
}

// Reciving UDP, with retry
fn udp_receive_ensure(socket: &UdpSocket) -> Option<ReliableUdpMsg> {

    let mut buffer = [0; 1024];

    match socket.recv_from(&mut buffer) {
        Ok((size, sender_addr)) => {
            if let Some(msg) = deserialize(&buffer[..size]) {
                if calc_checksum(&msg.data) == msg.header.checksum {
                    udp_ack(sender_addr); // Send ACK
                    return Some(msg);
                } 
                else{
                    udp_nak(sender_addr); // Send NAK
                }
            }
        }
        Err(e) => {
            eprintln!("Receive error: {}", e);
        }
    }
    return None;
}