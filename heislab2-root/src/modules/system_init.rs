

use std::env;                               //https://doc.rust-lang.org/std/env/index.html
use std::fs::File;                          //https://doc.rust-lang.org/std/fs/struct.File.html
use std::io::{BufRead, BufReader, Write};   //https://doc.rust-lang.org/std/io/trait.BufRead.html
use std::path::PathBuf;                     //https://doc.rust-lang.org/std/path/struct.PathBuf.html

use std::sync::{Mutex,Arc};
use std::time::{Duration,Instant};

use crate::modules::system_status::SystemState;
use crate::modules::udp_functions::udp::{UdpMsg,UdpHeader,UdpData,MessageType};

pub fn boot() -> SystemState {

    //Get config from "boot.txt"
    let (me_id_value, default_master_id) = load_config();

    //Just a dummy/filler message
    let mut starting_udpmsg =  UdpMsg {
        header: UdpHeader {
            sender_id: 0,
            message_type: MessageType::Worldview,
            checksum: vec![0],
        },
        data: UdpData::None,
    };

    // Set an old lifesign, this will trigger update of master
    let old_lifesign = Instant::now() - Duration::from_secs(10);

    // Generate a system state
    SystemState {
        me_id: me_id_value,
        master_id: Arc::new(Mutex::new(default_master_id)),
        last_lifesign: Arc::new(Mutex::new(old_lifesign)), 
        last_worldview: Arc::new(Mutex::new(starting_udpmsg)),
        active_elevators: Arc::new(Mutex::new(Vec::new())),
        all_orders: Arc::new(Mutex::new(Vec::new())),
        sent_messages: Arc::new(Mutex::new(Vec::new())),
    }
}

pub fn load_config() -> (u8, u8) {

    // Find "boot.txt" in the parentfolder of the program
    let exe_path = env::current_exe().expect("Failed to find path");
    let exe_dir = exe_path.parent().expect("Failed to get directory");
    let config_path: PathBuf = exe_dir.join("boot.txt");

    if config_path.exists() {
        let file = File::open(&config_path).expect("Couldn't open boot.txt");
        let reader = BufReader::new(file);

        let mut me_id = 0;
        let mut master_id = 0;

        for line in reader.lines() {
            let line = line.expect("Failed to read line");
            if let Some((key, value)) = line.split_once(':') {
                // match key to variable 
                match key.trim() {
                    "me_id" => me_id = value.trim().parse().unwrap_or(0),
                    "master_id" => master_id = value.trim().parse().unwrap_or(0),
                    _ => {}
                }
            }
        }

    return (me_id, master_id);
    }else{

        println!("Couldnt find boot.txt, using 5 and 6");
        return (5,6);
    }


}