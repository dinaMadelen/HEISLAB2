use tokio::time::{sleep, Duration};
use crossbeam_channel as cbc;
use tokio::process::Command;
use std::sync::atomic::Ordering;

use crate::elevator_logic::task_handler;
use crate::utils::SELF_ID;
use crate::world_view::world_view;
use crate::{config, utils::{self, print_ok}, world_view::world_view_update, elevio, elevio::poll::CallButton, elevio::elev as e};

use super::local_network;



struct LocalElevTxs {
    call_button: cbc::Sender<CallButton>,
    floor_sensor: cbc::Sender<u8>,
    stop_button: cbc::Sender<bool>,
    obstruction: cbc::Sender<bool>,
}

struct LocalElevRxs {
    call_button: cbc::Receiver<CallButton>,
    floor_sensor: cbc::Receiver<u8>,
    stop_button: cbc::Receiver<bool>,
    obstruction: cbc::Receiver<bool>,
}

struct LocalElevChannels {
    pub rxs: LocalElevRxs,
    pub txs: LocalElevTxs,
}

impl LocalElevChannels {
    pub fn new() -> Self {
        let (call_button_tx, call_button_rx) = cbc::unbounded::<elevio::poll::CallButton>();
        let (floor_sensor_tx, floor_sensor_rx) = cbc::unbounded::<u8>();
        let (stop_button_tx, stop_button_rx) = cbc::unbounded::<bool>();
        let (obstruction_tx, obstruction_rx) = cbc::unbounded::<bool>();

        LocalElevChannels { 
            rxs: LocalElevRxs { call_button: call_button_rx, floor_sensor: floor_sensor_rx, stop_button: stop_button_rx, obstruction: obstruction_rx }, 
            txs: LocalElevTxs { call_button: call_button_tx, floor_sensor: floor_sensor_tx, stop_button: stop_button_tx, obstruction: obstruction_tx } 
        }
    }
}


/// ### Henter ut lokal IP adresse
fn get_ip_address() -> String {
    let self_id = utils::SELF_ID.load(Ordering::SeqCst);
    format!("{}.{}", config::NETWORK_PREFIX, self_id)
}

/// ### Starter elevator_server
/// 
/// Tar høyde for om du er på windows eller ubuntu.
async fn start_elevator_server() {
    let ip_address = get_ip_address();
    let ssh_password = "Sanntid15"; // Hardkodet passord, vurder sikkerhetsrisiko

    if cfg!(target_os = "windows") {
        println!("Starter elevatorserver på Windows...");
        Command::new("cmd")
            .args(&["/C", "start", "elevatorserver"])
            .spawn()
            .expect("Failed to start elevator server");
    } else {
        println!("Starter elevatorserver på Linux...");
        
        let elevator_server_command = format!(
            "sshpass -p '{}' ssh student@{} 'nohup elevatorserver > /dev/null 2>&1 &'",
            ssh_password, ip_address
        );
        // Det starter serveren uten terminal. Om du vil avslutte serveren: pkill -f elevatorserver
        
        // Alternativt:                                                     pgrep -f elevatorserver  # Finner PID (Process ID)
        //                                                                  kill <PID>               # Avslutter prosessen


        println!("\nStarter elevatorserver i ny terminal:\n\t{}", elevator_server_command);

        let _ = Command::new("sh")
            .arg("-c")
            .arg(&elevator_server_command)
            .output().await
            .expect("Feil ved start av elevatorserver");
    }

    println!("Elevator server startet.");
}

/// ### Kjører den lokale heisen
pub async fn run_local_elevator(chs: local_network::LocalChannels) -> std::io::Result<()> {
    // Start elevator-serveren
    start_elevator_server().await;
    let local_elev_channels: LocalElevChannels = LocalElevChannels::new();
    utils::slave_sleep().await;
    let elevator: e::Elevator = e::Elevator::init(config::LOCAL_ELEV_IP, config::DEFAULT_NUM_FLOORS).expect("Feil!");
    
    // Start polling på meldinger fra heisen
    {
        let elevator = elevator.clone();
        tokio::spawn(async move {
            elevio::poll::call_buttons(elevator, local_elev_channels.txs.call_button, config::ELEV_POLL)
        });
    }
    {
        let elevator = elevator.clone();
        tokio::spawn(async move {
            elevio::poll::floor_sensor(elevator, local_elev_channels.txs.floor_sensor, config::ELEV_POLL)
        });
    }
    {
        let elevator = elevator.clone();
        tokio::spawn(async move {
            elevio::poll::stop_button(elevator, local_elev_channels.txs.obstruction, config::ELEV_POLL)
        });
    }
    {
        let elevator = elevator.clone();
        tokio::spawn(async move {
            elevio::poll::obstruction(elevator, local_elev_channels.txs.stop_button, config::ELEV_POLL)
        });
    }
 
    //Start en task som viderefører meldinger fra heisen til update_worldview 
    {
        let chs_clone = chs.clone();
        let _listen_task = tokio::spawn(async move {
            let _ = read_from_local_elevator(local_elev_channels.rxs, chs_clone).await;
        });
    }

    // Task som utfører deligerte tasks (ikke implementert korrekt enda)
    {
        let chs_clone = chs.clone();
        let _handle_task = tokio::spawn(async move {
            let _ = task_handler::execute_tasks(chs_clone, elevator).await;
        });
        tokio::task::yield_now().await;
    }
 
    // Loop som sender egen container på kanalen som motar slave-kontainere hvis man er master
    let mut wv = utils::get_wv(chs.clone());
    loop {
        utils::update_wv(chs.clone(), &mut wv).await;
        if utils::is_master(wv.clone()) {
            /* Oppdater task og task_status, send din container tilbake som om den fikk fra tcp */
            let wv_deser = world_view::deserialize_worldview(&world_view_update::join_wv(wv.clone(), wv.clone()));
            let self_idx = world_view::get_index_to_container(SELF_ID.load(Ordering::SeqCst), world_view::serialize_worldview(&wv_deser));
            if let Some(i) = self_idx {
                let _ = chs.mpscs.txs.container.send(world_view::serialize_elev_container(&wv_deser.elevator_containers[i])).await;
            }
        }
        sleep(config::TCP_PERIOD).await;
    }
}

/// ### Videresender melding fra egen heis til update_wv
async fn read_from_local_elevator(rxs: LocalElevRxs, chs: local_network::LocalChannels) -> std::io::Result<()> {
    loop {
        // Sjekker hver kanal med `try_recv()`
        if let Ok(call_button) = rxs.call_button.try_recv() {
            //println!("CB: {:#?}", call_button);
            let msg = local_network::ElevMessage {
                msg_type: local_network::ElevMsgType::CBTN,
                call_button: Some(call_button),
                floor_sensor: None,
                stop_button: None,
                obstruction: None,
            };
            let _ = chs.mpscs.txs.local_elev.send(msg).await;
        }

        if let Ok(floor) = rxs.floor_sensor.try_recv() {
            //println!("Floor: {:#?}", floor);
            let msg = local_network::ElevMessage {
                msg_type: local_network::ElevMsgType::FSENS,
                call_button: None,
                floor_sensor: Some(floor),
                stop_button: None,
                obstruction: None,
            };
            let _ = chs.mpscs.txs.local_elev.send(msg).await;
        }

        if let Ok(stop) = rxs.stop_button.try_recv() {
            //println!("Stop button: {:#?}", stop);
            let msg = local_network::ElevMessage {
                msg_type: local_network::ElevMsgType::SBTN,
                call_button: None,
                floor_sensor: None,
                stop_button: Some(stop),
                obstruction: None,
            };
            let _ = chs.mpscs.txs.local_elev.send(msg).await;
        }

        if let Ok(obstr) = rxs.obstruction.try_recv() {
            //println!("Obstruction: {:#?}", obstr);
            let msg = local_network::ElevMessage {
                msg_type: local_network::ElevMsgType::OBSTRX,
                call_button: None,
                floor_sensor: None,
                stop_button: None,
                obstruction: Some(obstr),
            };
            let _ = chs.mpscs.txs.local_elev.send(msg).await;
        }

        // Kort pause for å unngå å spinne CPU unødvendig
        sleep(Duration::from_millis(10)).await;
    }
}










