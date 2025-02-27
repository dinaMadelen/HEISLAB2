//UDP Functions for sending and reciving data over UDP

/*----------------------Left to IMPLEMENT:

Sequence number for correct order, Threading ,Mutex

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
udp_send_retry        same as send, but requrires ACK

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
9:  Error: Can't service queue/ Going Offline (Master/Slave)
10: Error: Any

----------------------------------------------- !!!OBS!!! ADD TO Cargo.toml: 

[dependencies]
serde = { version = "1", features = ["derive"] }
bincode = "1"
sha2 = { version = "0.11.0-pre.4" }

*/

/* ---------------------------------------------------------Temporary for testing */


// Possible states for the elevator
enum Status{
    Idle,
    Moving,
    Maintenance,
    Error
}


// Elevator 
struct Elevator{

    id:i8,
    current_floor:i8,
    going_up:bool,
    queue:Vec<i8>,
    status:Status
}


// Functions for elevator struct
impl Elevator{

    // Add a floor to the queuem then sorts the queue. 
    fn add_to_queue(&mut self, floor: i8) {
        if !self.queue.contains(&floor) {
            self.queue.push(floor);
            self.queue = self.sort_queue();
        }
        else{
            self.send_status();
        }
    }
    

    // Sets current status (Enum Status) for elevator,
    fn set_status(&mut self ,status:Status){

        match status{

            Status::Maintenance => {
                self.status = Status::Maintenance;
                self.queue.clear();
            }

            // Floors are read as i8, direction true is going up, false is going down.
            Status::Moving => {
                if self.queue.is_empty(){

                }else{

                    if *self.queue.first().unwrap() < self.current_floor { // Get floor in queue or floor out of bounds if empty
                        self.going_up = false; 
                        self.current_floor = *self.queue.first().unwrap();
                        self.queue.remove(0);

                    } else{
                        self.going_up = true;
                        self.current_floor = *self.queue.first().unwrap();
                        self.queue.remove(0);
                    }
                }
            }

            Status::Idle => {
                self.status = Status::Idle;
                self.going_up = true; //Going up is default, maybe just leave in current state or add Enum for none?
            }

            Status::Error => {
                self.status = Status::Error;
                self.queue.clear();
                self.send_status();
            }
        }    
    }

    fn sort_queue(&self) -> Vec<i8> {
        let (mut non_negative, mut negative): (Vec<i8>, Vec<i8>) = <Vec<i8> as Clone>::clone(&self.queue)
            .into_iter()
            .partition(|&x| x >= 0);
    
        non_negative.sort();
        negative.sort();
    
        // Non-negative numbers first, negative numbers last
        non_negative.extend(negative);

        let (mut infront, mut behind): (Vec<i8>, Vec<i8>) = non_negative
        .into_iter()
        .partition(|&x| x <= self.current_floor); //split at current floor

        infront.extend(behind);// add passed floors at back of queue (add back in oposite direciton?)
        return infront;
    }

    // Moves to next floor, if empty queue, set status to idle.
    fn go_next_floor(&mut self) {
        if let Some(next_floor) = self.queue.first() {
            if *next_floor > self.current_floor {
                self.going_up = true;
                self.current_floor += 1;
                self.set_status(Status::Moving);
            } else if *next_floor < self.current_floor {
                self.going_up = false;
                self.current_floor -= 1;
                self.set_status(Status::Moving);
            } else {
                self.going_up = true; //Default direction is up
            }
        }
        else {
            self.set_status(Status::Idle);
        }
    }

    fn send_status(&self){
        todo!("Implement send status function");
    }

}

//----------------------------END TEMP--------------------------------------------------------------

//----------------------------------------------Imports
use std::net::{UdpSocket, SocketAddr};  // https://doc.rust-lang.org/std/net/struct.UdpSocket.html
//use std::sync::{Arc, Mutex};          // https://doc.rust-lang.org/std/sync/struct.Mutex.html
use serde::{Serialize, Deserialize};    // https://serde.rs/impl-serialize.html         //Add to Cargo.toml file, Check comment above
                                        // https://docs.rs/serde/latest/serde/ser/trait.Serialize.html#tymethod.serialize
use bincode;                            // https://docs.rs/bincode/latest/bincode/      //Add to Cargo.toml file, Check comment above
use sha2::{Sha256, Digest};             // https://docs.rs/sha2/latest/sha2/            //Add to Cargo.toml file, Check comment above
use std::time::Duration;                    // https://doc.rust-lang.org/std/time/struct.Duration.html


//----------------------------------------------Enum
#[derive(Debug, Serialize, Deserialize, Clone)]
enum message_type{

Wordview,
Ack,
Nak,
New_Order,
New_Master,
New_Online,
Request_Queue,
Respond_Queue,
Error_Worldview,
Respond_Er_Worldview,
Error_Offline,
Request_Resend,
}

//----------------------------------------------Structs

#[derive(Debug, Serialize, Deserialize, Clone)] // this is needed to serialize message
//UDP Header
struct UdpHeader{
    sender_id: i8,                  // ID of the sender of the message.
    message_id: message_type,       // ID for what kind of message it is, e.g. Button press, or Update queue.
    sequence_number: u32,           // Number of message in order.
    checksum: Vec<u8>,              // Hash of data to check message integrity.
}

#[derive(Debug, Serialize, Deserialize, Clone)] // this is needed to serialize message
//UDP Message Struct
struct UdpMsg{
    header: UdpHeader,              // Header struct containing information about the message itself
    data: Vec<u8>,                  // Data so be sent.
}

//----------------------------------------------Functions

fn make_Udp_msg(elevator: Elevator,message_type: message_type, message:Vec<u8>) ->UdpMsg{

   let hash = calc_checksum(&message);
   let mut overhead = UdpHeader{sender_id:elevator.id ,message_id:message_type,sequence_number:0, checksum:hash};  
   let msg = UdpMsg{header: overhead, data:message};
   return msg;

}

// Split UdpMsg into bytes
fn serialize(msg: &UdpMsg) -> Vec<u8> {

    let serialized_msg = bincode::serialize(msg).expect("Failed to serialize message");
    return serialized_msg;
}

// Combine bytes in message buffer into UdpMsg 
fn deserialize(buffer: &[u8]) -> Option<UdpMsg>{

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

// Compare checksums, Not sure if we need this or not
fn comp_checksum(msg: &UdpMsg)-> bool{

    return calc_checksum(&msg.data) == msg.header.checksum;
}

//Recive UDP message
fn udp_recive(socket: &UdpSocket,max_wait :u8) -> Option<UdpMsg>{

    socket.set_read_timeout(Some(Duration::new(max_wait.into(), 0)))
    .expect(&format!("Failed to set read timeout of {}s", max_wait));

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
fn udp_send_ensure(socket: &UdpSocket, target_addr: &str, msg: &UdpMsg, max_retry: u8) -> bool {

    let data = serialize(msg);
    let mut retries = max_retry;

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
                if buffer[0] == 0x06 {                                   // ACK received, ASCII for ACK
                    println!("ACK received for {}", msg.header.sequence_number);
                    return true;                                         // Message sucessfully sent and recived
                }
            }
            _=> retries -= 1,  // Anything other than an empty or accepted message
        }
    }

    println!("Failed to send after retries.");
    return false; 
}

// Reciving UDP, with ACK
fn udp_receive_ensure(socket: &UdpSocket, max_wait: u8) -> Option<UdpMsg> {

    socket.set_read_timeout(Some(Duration::new(max_wait.into(), 0))).expect("Failed to set read timeout");
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



//------------------------------Tests-----------------------

#[cfg(test)] // https://doc.rust-lang.org/book/ch11-03-test-organization.html Run tests with "cargo test"
mod tests {
    use super::*;
    use std::net::UdpSocket;

    #[test]
    fn test_serialize_deserialize() {
        let msg = UdpMsg {
            header: UdpHeader {
                sender_id: 1,
                message_id: message_type::Ack,
                sequence_number: 0,
                checksum: vec![0x12, 0x34],
            },
            data: vec![1, 2, 3, 4],
        };

        let serialized = serialize(&msg);
        let deserialized = deserialize(&serialized).expect("Deserialization failed");

        assert_eq!(msg.header.sender_id, deserialized.header.sender_id);
        assert_eq!(msg.header.sequence_number, deserialized.header.sequence_number);
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
                sequence_number: 0,
                checksum,
            },
            data,
        };
        assert!(comp_checksum(&msg));
    }

    #[test]
    fn test_udp_send_recv() {
        let socket = UdpSocket::bind("127.0.0.1:0").expect("Failed to bind socket");
        let local_addr = socket.local_addr().expect("Failed to get socket address");
            
        let msg = UdpMsg {
            header: UdpHeader {
                sender_id: 1,
                message_id: message_type::Ack,
                sequence_number: 0,
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
            
        let received_msg = udp_recive(&recv_socket, 5).expect("Failed to receive message");
        assert_eq!(msg.data, received_msg.data);
    }
}



fn main(){


}
