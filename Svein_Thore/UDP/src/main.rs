//UDP Functions for sending and reciving data over UDP

/*----------------------Left to IMPLEMENT:

Sequence number for correct order, Threading ,Mutex

------------------------Structs in this file:

UdpMsg                Contains message data and overhead
UdpHeader             Contains overhead

-----------------------------------------------Functions in this file:
serilize               UdpMsg -> Vec<u8>
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
8:  Response to  Error: Worldview hash (Master responds with queues)
9:  Error: Can't service queue (Master/Slave)
10: Error: 

----------------------------------------------- !!!OBS!!! ADD TO Cargo.toml: 

[dependencies]
serde = { version = "1", features = ["derive"] }
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
//use std::sync::{Arc, Mutex};            // https://doc.rust-lang.org/std/sync/struct.Mutex.html
use serde::{Serialize, Deserialize};    // https://serde.rs/impl-serialize.html  
                                        // https://docs.rs/serde/latest/serde/ser/trait.Serialize.html#tymethod.serialize
use bincode;                            // https://docs.rs/bincode/latest/bincode/      //Add to Cargo.toml file, Check comment above
use sha2::{Sha256, Digest};             // https://docs.rs/sha2/latest/sha2/            //Add to Cargo.toml file, Check comment above

//----------------------------------------------Struct

#[derive(Debug, Serialize, Deserialize)]
//UDP Header
struct UdpHeader{
    sender_id: u8,      // ID of the sender of the message.
    message_id: u8,     // ID for what kind of message it is, e.g. Button press, or Update queue.
    sequence_number: u32, // Number of message in order.
    checksum: Vec<u8>,  // Hash of data to check message integrity.
}

#[derive(Debug, Serialize, Deserialize)]
//UDP Message Struct
struct UdpMsg{
    header: UdpHeader, // Header struct containing information about the message itself
    data: Vec<u8>,      // Data so be sent.
}

//----------------------------------------------Functions

// Split UdpMsg into bytes
fn serialize(msg: &UdpMsg) -> Vec<u8> {

    let serialized_msg = bincode::serialize(msg).expect("Failed to serialize message");
    return serialized_msg;

}

// Combine bytes into UdpMsg 
fn deserialize(buffer: &[u8]) -> Option<UdpMsg> {

    let deserialized_msg = bincode::deserialize(buffer).ok(); 
    return deserialized_msg;
}

// Calculate Checksum. 
fn calc_checksum(data: &Vec<u8>) -> Vec<u8>{

    let mut hasher = Sha256::new();
    hasher.update(data);
    let hash = hasher.finalize();
    return hash.to_vec(); 
}

// Compare checksums, Not sure if i need this or not
fn comp_checksum(msg: &UdpMsg)-> bool{

    return calc_checksum(&msg.data) == msg.header.checksum;
    }

//Recive UDP message
fn udp_recive(socket: &UdpSocket) -> Option<UdpMsg>{

    let mut buffer = [0;1024];
    //let _lock = res_mutex_recv.lock().unwrap();
    //Recive message
    match socket.recv_from(&mut buffer){
 
        Ok((size,sender)) => {
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
fn udp_ack(socket: &UdpSocket,target_address: SocketAddr)-> bool{

    match socket.send_to(&[0x06] , target_address) {
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
fn udp_nak(socket: &UdpSocket,target_address: SocketAddr)-> bool{

    match socket.send_to(&[0x15] , target_address) {
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
fn udp_send(socket: &UdpSocket,target_adress: SocketAddr,msg: &UdpMsg)->bool{

    let data = serialize(msg);
    match socket.send_to(&data,target_adress){
        Ok(_) => {
            println!("Message sent to: {}",target_adress);
            return true;
        }
        Err(e) => {
            eprintln!("Error sending message: {}",e);
            return false;
        }
    }
}

//Broadcast
fn udp_broadcast(msg:&UdpMsg){

    let socket = UdpSocket::bind("0.0.0.0:0").expect("Failed to bind socket, broadcast");
    socket.set_broadcast(true).expect("failed to activate broadcast");

    let msg = serialize(msg);
    let target_address = "255.255.255.255;20000";

    match socket.send_to(&msg,target_address) {
        Ok(_) => {
            println!("Broadcast successful");
        }
        Err(e) => {
            eprintln!("Error sending data: {}", e);
        }
    }
}

// Sending UDP, with retry
fn udp_send_ensure(socket: &UdpSocket, target_addr: &str, msg: &UdpMsg) -> bool {

    let data = serialize(msg);
    let mut retries = 5;

    while retries > 0 {

        match socket.send_to(&data,target_addr){
            Ok(_) => {
                println!("Message sent to: {}",target_addr);
            }
            Err(e) => {
                eprintln!("Error sending message: {}",e);
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
            _=> retries -= 1,  // Anything other than an empty or accepted message
        }
    }

    println!("Failed to send after retries.");
    return false; 
}

// Reciving UDP, with ensure
fn udp_receive_ensure(socket: &UdpSocket) -> Option<UdpMsg> {

    let mut buffer = [0; 1024];

    match socket.recv_from(&mut buffer) {
        Ok((size, sender_addr)) => {
            if let Some(msg) = deserialize(&buffer[..size]) {
                if calc_checksum(&msg.data) == msg.header.checksum {
                    udp_ack(socket,sender_addr); // Send ACK
                    return Some(msg);
                } 
                else{
                    udp_nak(socket,sender_addr); // Send NAK
                }
            }
        }
        Err(e) => {
            eprintln!("error reciving{}",e);
        }
    }
    return None;
}

fn main(){
       println!("------------------------------OK! det funker, eller kompilerer ihverfall :P");
}
