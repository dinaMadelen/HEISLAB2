//TCP


use std::io::{Read, Write};
use std::net::{TcpStream}; //https://doc.rust-lang.org/std/net/struct.TcpStream.html
use std::thread;
use std::time::Duration;


fn main(){

    let server_name: &str = "127.0.0.1:34933";

    let mut stream = TcpStream::connect(&server_name).expect("Couldn't connect to server");

    println!("Connected to server at {}",&server_name);

    let mut i: u8 = 0;
    loop {
        i += 1;
        if i == u8::MAX {
        i = 0;
        }

        let msg = format!("Number: {}\0", i);
        match stream.write(msg.as_bytes()) {
            Ok(_) => println!("Sent Message: {}", msg),
            Err(e) => println!("Error sending data: {}", e),
        }

        let mut buffer = [0; 1024];
        match stream.read(&mut buffer) {

            Ok(size) => {
                if size > 0{ 
                    if let Ok(message) = std::str::from_utf8(&buffer[..size]) {
                        println!("Response: {}", message.trim_end_matches('\0'));
                    } else {
                        println!("Received non-UTF8 data: {:?}", &buffer[..size]);
                    }
                }
            }
            Err(e) => {
                println!("Error receiving data: {}", e)
            }
        }
        thread::sleep(Duration::from_millis(500));

    }


}
