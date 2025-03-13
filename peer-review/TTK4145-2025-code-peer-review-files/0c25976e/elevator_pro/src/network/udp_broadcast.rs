//! ## Håndterer UDP-logikk i systemet

use crate::config;
use crate::utils;
use super::local_network;

use std::net::SocketAddr;
use std::sync::atomic::Ordering;
use std::sync::OnceLock;
use std::sync::atomic::AtomicBool;
use std::thread::sleep;
use std::time::Duration;
use tokio::net::UdpSocket;
use socket2::{Domain, Socket, Type};
use std::borrow::Cow;

static UDP_TIMEOUT: OnceLock<AtomicBool> = OnceLock::new(); // worldview_channel_request
pub fn get_udp_timeout() -> &'static AtomicBool {
    UDP_TIMEOUT.get_or_init(|| AtomicBool::new(false))
}

/// ### Starter og kjører udp-broadcaster
pub async fn start_udp_broadcaster(mut chs: local_network::LocalChannels) -> tokio::io::Result<()> {
    // Sett opp sockets
    chs.subscribe_broadcast();
    let addr: &str = &format!("{}:{}", config::BC_ADDR, config::DUMMY_PORT);
    let addr2: &str = &format!("{}:0", config::BC_LISTEN_ADDR);

    let broadcast_addr: SocketAddr = addr.parse().expect("ugyldig adresse"); // UDP-broadcast adresse
    let socket_addr: SocketAddr = addr2.parse().expect("Ugyldig adresse");
    let socket = Socket::new(Domain::IPV4, Type::DGRAM, None)?;
    
    
    socket.set_reuse_address(true)?;
    socket.set_broadcast(true)?;
    socket.bind(&socket_addr.into())?;
    let udp_socket = UdpSocket::from_std(socket.into())?;

    let mut wv = utils::get_wv(chs.clone());
    loop{
        let chs_clone = chs.clone();
        utils::update_wv(chs_clone, &mut wv).await;

        // Hvis du er master, broadcast worldview
        if utils::SELF_ID.load(Ordering::SeqCst) == wv[config::MASTER_IDX] {
            //TODO: Lag bedre delay?
            sleep(config::UDP_PERIOD);
            let mesage = format!("{:?}{:?}", config::KEY_STR, wv).to_string();
            udp_socket.send_to(mesage.as_bytes(), &broadcast_addr).await?;
        }
    }
}

/// ### Starter og kjører udp-listener
pub async fn start_udp_listener(mut chs: local_network::LocalChannels) -> tokio::io::Result<()> {
    //Sett opp sockets
    chs.subscribe_broadcast();
    let self_id = utils::SELF_ID.load(Ordering::SeqCst);
    let broadcast_listen_addr = format!("{}:{}", config::BC_LISTEN_ADDR, config::DUMMY_PORT);
    let socket_addr: SocketAddr = broadcast_listen_addr.parse().expect("Ugyldig adresse");
    let socket_temp = Socket::new(Domain::IPV4, Type::DGRAM, None)?;
    
    
    socket_temp.set_reuse_address(true)?;
    socket_temp.set_broadcast(true)?;
    socket_temp.bind(&socket_addr.into())?;
    let socket = UdpSocket::from_std(socket_temp.into())?;
    let mut buf = [0; config::UDP_BUFFER];
    let mut read_wv: Vec<u8> = Vec::new();
    
    let mut message: Cow<'_, str> = std::borrow::Cow::Borrowed("a");
    let mut my_wv = utils::get_wv(chs.clone());
    // Loop mottar og behandler udp-broadcaster
    loop {
        match socket.recv_from(&mut buf).await {
            Ok((len, _)) => {
                message = String::from_utf8_lossy(&buf[..len]);
                // println!("WV length: {:?}", len);
            }
            Err(e) => {
                // utils::print_err(format!("udp_broadcast.rs, udp_listener(): {}", e));
                return Err(e);
            }
        }
        
        // Verifiser at broadcasten var fra 'oss'
        if &message[1..config::KEY_STR.len()+1] == config::KEY_STR { //Plusser på en, siden serialiseringa av stringen tar med '"'-tegnet
            let clean_message = &message[config::KEY_STR.len()+3..message.len()-1]; // Fjerner `"`
            read_wv = clean_message
            .split(", ") // Del opp på ", "
            .filter_map(|s| s.parse::<u8>().ok()) // Konverter til u8, ignorer feil
            .collect(); // Samle i Vec<u8>

            utils::update_wv(chs.clone(), &mut my_wv).await;
            if read_wv[config::MASTER_IDX] != my_wv[config::MASTER_IDX] {
                // mulighet for debug print
            } else {
                // Betyr at du har fått UDP-fra nettverkets master -> Restart UDP watchdog
                get_udp_timeout().store(false, Ordering::SeqCst);
                // println!("Resetter UDP-watchdog");
            }

            // Hvis broadcast har lavere ID enn nettverkets tidligere master
            if my_wv[config::MASTER_IDX] >= read_wv[config::MASTER_IDX] {
                if !(self_id == read_wv[config::MASTER_IDX]) {
                    //Oppdater egen WV
                    my_wv = read_wv;
                    let _ = chs.mpscs.txs.udp_wv.send(my_wv.clone()).await;
                }
            }
            
        }
    }
}


/// ### jalla udp watchdog
pub async fn udp_watchdog(chs: local_network::LocalChannels) {
    loop {
        if get_udp_timeout().load(Ordering::SeqCst) == false {
            get_udp_timeout().store(true, Ordering::SeqCst);
            tokio::time::sleep(Duration::from_millis(1000)).await;
        }
        else {
            get_udp_timeout().store(false, Ordering::SeqCst); //resetter watchdogen
            utils::print_warn("UDP-watchdog: Timeout".to_string());
            let _ = chs.mpscs.txs.tcp_to_master_failed.send(true).await;
        }
    }
}

