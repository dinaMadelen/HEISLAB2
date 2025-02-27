use std::net::{UdpSocket};              
use std::time::Duration;
use std::process::Command; // https://doc.rust-lang.org/std/process/struct.Command.html
use std::thread::sleep;

fn main() {
    let socket = UdpSocket::bind("0.0.0.0:30000").expect("Could not bind UDP socket");
    socket.set_read_timeout(Some(Duration::new(5, 0))).expect("Failed to set read timeout");

    let last_received: i32 = 0;

    let mut buffer = [0u8; 1024];
    
        //Handle first message then start loop
    if let Ok((num_bytes, sender_addr)) = socket.recv_from(&mut buffer) {
        println!("Sender is alive, booting as listner");
        let received_number = String::from_utf8_lossy(&buffer[..num_bytes]).parse::<i32>().unwrap_or(0);
        println!("Received: {} from {} <- First message", received_number, sender_addr);
    
        listen_loop(socket, last_received);
    } else {
        
        //No message recived, boot as sender
        print!("Starting program.\nNo sender found: Setting role to sender\nDropping socket for reciving\n");
        drop(socket); 
        sleep(Duration::from_millis(50));   // Wait for OS to release the port
        assume_primary(last_received);      // Start new instance of listner with the last recived message as startingref
                                            // and start swap to sending
    }
}

// Listen for messages
fn listen_loop(socket: UdpSocket,mut last_received: i32) {
    let mut buffer = [0u8; 1024];

    loop {
        if let Ok((num_bytes, sender_addr)) = socket.recv_from(&mut buffer) {
            let received_number = String::from_utf8_lossy(&buffer[..num_bytes]).parse::<i32>().unwrap_or(0);
            println!("Received: {} from {}", received_number, sender_addr);
            last_received= received_number;
        } else {
            println!("Sender dead: Swapping role to sender");
            drop(socket);
            sleep(Duration::from_millis(100)); // Wait for PORT to be released, to avoid port conflict
            assume_primary(last_received);
            break;
        }
    }
}

// Change role to sender and spawn a new instance
fn assume_primary(last_received: i32) {
    let mut value = last_received as u8;

    println!("Swapping role to sender, Starting value: {}", value);

    let socket = UdpSocket::bind("0.0.0.0:0").expect("Could not bind sender socket");

    match std::env::consts::OS{

        "windows"=>{
        Command::new("cmd")  // https://doc.rust-lang.org/std/process/struct.Command.html
        .args(&["/C", "start", std::env::current_exe().unwrap().to_str().unwrap()])
        .spawn()
        .expect("Failed to start a backup");
        }

        "macos" | "linux" => {
            Command::new("sh")  
                .arg("-c")
                .arg(format!("{} &", std::env::current_exe().unwrap().to_str().unwrap()))
                .spawn()
                .expect("Failed to start a backup instance");
        }

        _=>{
            println!("Failed to start backup: Cant find OS,{}",std::env::consts::OS)
        }


    }


    sleep(Duration::from_millis(2000)); // Wait for new listener to boot before sending 
    loop {
    let message = value.to_string();
    socket.send_to(message.as_bytes(), "127.0.0.1:30000").expect("Failed to send number");
    println!("Sent: {}", value);
    if value == u8::MAX{
        value=u8::MIN;
        println!("Value(u8) maxed out, resetting counter");
    }
    else{
        value += 1;
    }
    sleep(Duration::from_millis(1000));
    }
    

}