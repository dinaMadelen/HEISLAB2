use crate::{elevio::poll::CallButton, world_view::world_view::TaskStatus};
use tokio::sync::{mpsc, broadcast, watch, Semaphore};
use std::sync::Arc;
use crate::world_view::world_view::Task;


#[derive(Debug)]
pub enum ElevMsgType {
    CBTN,
    FSENS,
    SBTN,
    OBSTRX,
}

#[derive(Debug)]
pub struct ElevMessage {
    pub msg_type: ElevMsgType,
    pub call_button: Option<CallButton>,
    pub floor_sensor: Option<u8>,
    pub stop_button: Option<bool>,
    pub obstruction: Option<bool>,
}


// --- MPSC-KANALAR ---

pub struct MpscTxs {
    pub udp_wv: mpsc::Sender<Vec<u8>>,
    pub tcp_to_master_failed: mpsc::Sender<bool>,
    pub container: mpsc::Sender<Vec<u8>>,
    pub remove_container: mpsc::Sender<u8>,
    pub local_elev: mpsc::Sender<ElevMessage>,
    pub sent_tcp_container: mpsc::Sender<Vec<u8>>,

    // 10 nye buffer-kanalar
    pub new_task: mpsc::Sender<(Task, u8, CallButton)>,
    pub update_task_status: mpsc::Sender<(u16, TaskStatus)>,
    pub mpsc_buffer_ch2: mpsc::Sender<Vec<u8>>,
    pub mpsc_buffer_ch3: mpsc::Sender<Vec<u8>>,
    pub mpsc_buffer_ch4: mpsc::Sender<Vec<u8>>,
    pub mpsc_buffer_ch5: mpsc::Sender<Vec<u8>>,
    pub mpsc_buffer_ch6: mpsc::Sender<Vec<u8>>,
    pub mpsc_buffer_ch7: mpsc::Sender<Vec<u8>>,
    pub mpsc_buffer_ch8: mpsc::Sender<Vec<u8>>,
    pub mpsc_buffer_ch9: mpsc::Sender<Vec<u8>>,
}

pub struct MpscRxs {
    pub udp_wv: mpsc::Receiver<Vec<u8>>,
    pub tcp_to_master_failed: mpsc::Receiver<bool>,
    pub container: mpsc::Receiver<Vec<u8>>,
    pub remove_container: mpsc::Receiver<u8>,
    pub local_elev: mpsc::Receiver<ElevMessage>,
    pub sent_tcp_container: mpsc::Receiver<Vec<u8>>,

    // 10 nye buffer-kanalar
    pub new_task: mpsc::Receiver<(Task, u8, CallButton)>,
    pub update_task_status: mpsc::Receiver<(u16, TaskStatus)>,
    pub mpsc_buffer_ch2: mpsc::Receiver<Vec<u8>>,
    pub mpsc_buffer_ch3: mpsc::Receiver<Vec<u8>>,
    pub mpsc_buffer_ch4: mpsc::Receiver<Vec<u8>>,
    pub mpsc_buffer_ch5: mpsc::Receiver<Vec<u8>>,
    pub mpsc_buffer_ch6: mpsc::Receiver<Vec<u8>>,
    pub mpsc_buffer_ch7: mpsc::Receiver<Vec<u8>>,
    pub mpsc_buffer_ch8: mpsc::Receiver<Vec<u8>>,
    pub mpsc_buffer_ch9: mpsc::Receiver<Vec<u8>>,
}

impl Clone for MpscTxs {
    fn clone(&self) -> MpscTxs {
        MpscTxs {
            udp_wv: self.udp_wv.clone(),
            tcp_to_master_failed: self.tcp_to_master_failed.clone(),
            container: self.container.clone(),
            remove_container: self.remove_container.clone(),
            local_elev: self.local_elev.clone(),
            sent_tcp_container: self.sent_tcp_container.clone(),

            // Klonar buffer-kanalane
            new_task: self.new_task.clone(),
            update_task_status: self.update_task_status.clone(),
            mpsc_buffer_ch2: self.mpsc_buffer_ch2.clone(),
            mpsc_buffer_ch3: self.mpsc_buffer_ch3.clone(),
            mpsc_buffer_ch4: self.mpsc_buffer_ch4.clone(),
            mpsc_buffer_ch5: self.mpsc_buffer_ch5.clone(),
            mpsc_buffer_ch6: self.mpsc_buffer_ch6.clone(),
            mpsc_buffer_ch7: self.mpsc_buffer_ch7.clone(),
            mpsc_buffer_ch8: self.mpsc_buffer_ch8.clone(),
            mpsc_buffer_ch9: self.mpsc_buffer_ch9.clone(),
        }
    }
}

pub struct Mpscs {
    pub txs: MpscTxs,
    pub rxs: MpscRxs,
}

impl Mpscs {
    pub fn new() -> Self {
        let (tx_udp, rx_udp) = mpsc::channel(300);
        let (tx1, rx1) = mpsc::channel(300);
        let (tx2, rx2) = mpsc::channel(300);
        let (tx3, rx3) = mpsc::channel(300);
        let (tx4, rx4) = mpsc::channel(300);
        let (tx5, rx5) = mpsc::channel(300);

        // Initialisering av 10 nye buffer-kanalar
        let (tx_buf0, rx_buf0) = mpsc::channel(300);
        let (tx_buf1, rx_buf1) = mpsc::channel(300);
        let (tx_buf2, rx_buf2) = mpsc::channel(300);
        let (tx_buf3, rx_buf3) = mpsc::channel(300);
        let (tx_buf4, rx_buf4) = mpsc::channel(300);
        let (tx_buf5, rx_buf5) = mpsc::channel(300);
        let (tx_buf6, rx_buf6) = mpsc::channel(300);
        let (tx_buf7, rx_buf7) = mpsc::channel(300);
        let (tx_buf8, rx_buf8) = mpsc::channel(300);
        let (tx_buf9, rx_buf9) = mpsc::channel(300);

        Mpscs {
            txs: MpscTxs {
                udp_wv: tx_udp,
                tcp_to_master_failed: tx1,
                container: tx2,
                remove_container: tx3,
                local_elev: tx4,
                sent_tcp_container: tx5,

                // Legg til dei nye buffer-kanalane
                new_task: tx_buf0,
                update_task_status: tx_buf1,
                mpsc_buffer_ch2: tx_buf2,
                mpsc_buffer_ch3: tx_buf3,
                mpsc_buffer_ch4: tx_buf4,
                mpsc_buffer_ch5: tx_buf5,
                mpsc_buffer_ch6: tx_buf6,
                mpsc_buffer_ch7: tx_buf7,
                mpsc_buffer_ch8: tx_buf8,
                mpsc_buffer_ch9: tx_buf9,
            },
            rxs: MpscRxs {
                udp_wv: rx_udp,
                tcp_to_master_failed: rx1,
                container: rx2,
                remove_container: rx3,
                local_elev: rx4,
                sent_tcp_container: rx5,

                // Legg til dei nye buffer-kanalane
                new_task: rx_buf0,
                update_task_status: rx_buf1,
                mpsc_buffer_ch2: rx_buf2,
                mpsc_buffer_ch3: rx_buf3,
                mpsc_buffer_ch4: rx_buf4,
                mpsc_buffer_ch5: rx_buf5,
                mpsc_buffer_ch6: rx_buf6,
                mpsc_buffer_ch7: rx_buf7,
                mpsc_buffer_ch8: rx_buf8,
                mpsc_buffer_ch9: rx_buf9,
            },
        }
    }
}

impl Clone for Mpscs {
    fn clone(&self) -> Mpscs {
        let (_, rx_udp) = mpsc::channel(300);
        let (_, rx1) = mpsc::channel(300);
        let (_, rx2) = mpsc::channel(300);
        let (_, rx3) = mpsc::channel(300);
        let (_, rx4) = mpsc::channel(300);
        let (_, rx5) = mpsc::channel(300);

        // Initialiser mottakar-kanalane ved cloning
        let (_, rx_buf0) = mpsc::channel(300);
        let (_, rx_buf1) = mpsc::channel(300);
        let (_, rx_buf2) = mpsc::channel(300);
        let (_, rx_buf3) = mpsc::channel(300);
        let (_, rx_buf4) = mpsc::channel(300);
        let (_, rx_buf5) = mpsc::channel(300);
        let (_, rx_buf6) = mpsc::channel(300);
        let (_, rx_buf7) = mpsc::channel(300);
        let (_, rx_buf8) = mpsc::channel(300);
        let (_, rx_buf9) = mpsc::channel(300);

        Mpscs {
            txs: self.txs.clone(),
            rxs: MpscRxs {
                udp_wv: rx_udp,
                tcp_to_master_failed: rx1,
                container: rx2,
                remove_container: rx3,
                local_elev: rx4,
                sent_tcp_container: rx5,

                // Klonar buffer-kanalane
                new_task: rx_buf0,
                update_task_status: rx_buf1,
                mpsc_buffer_ch2: rx_buf2,
                mpsc_buffer_ch3: rx_buf3,
                mpsc_buffer_ch4: rx_buf4,
                mpsc_buffer_ch5: rx_buf5,
                mpsc_buffer_ch6: rx_buf6,
                mpsc_buffer_ch7: rx_buf7,
                mpsc_buffer_ch8: rx_buf8,
                mpsc_buffer_ch9: rx_buf9,
            },
        }
    }
}


// --- BROADCAST-KANALAR ---

pub struct BroadcastTxs {
    pub shutdown: broadcast::Sender<()>,
    pub broadcast_buffer_ch1: broadcast::Sender<bool>,
    pub broadcast_buffer_ch2: broadcast::Sender<bool>,
    pub broadcast_buffer_ch3: broadcast::Sender<bool>,
    pub broadcast_buffer_ch4: broadcast::Sender<bool>,
    pub broadcast_buffer_ch5: broadcast::Sender<bool>,
}

pub struct BroadcastRxs {
    pub shutdown: broadcast::Receiver<()>,
    pub broadcast_buffer_ch1: broadcast::Receiver<bool>,
    pub broadcast_buffer_ch2: broadcast::Receiver<bool>,
    pub broadcast_buffer_ch3: broadcast::Receiver<bool>,
    pub broadcast_buffer_ch4: broadcast::Receiver<bool>,
    pub broadcast_buffer_ch5: broadcast::Receiver<bool>,
}

impl Clone for BroadcastTxs {
    fn clone(&self) -> BroadcastTxs {
        BroadcastTxs {
            shutdown: self.shutdown.clone(),
            broadcast_buffer_ch1: self.broadcast_buffer_ch1.clone(),
            broadcast_buffer_ch2: self.broadcast_buffer_ch2.clone(),
            broadcast_buffer_ch3: self.broadcast_buffer_ch3.clone(),
            broadcast_buffer_ch4: self.broadcast_buffer_ch4.clone(),
            broadcast_buffer_ch5: self.broadcast_buffer_ch5.clone(),
        }
    }
}

impl BroadcastTxs {
    pub fn subscribe(&self) -> BroadcastRxs {
        BroadcastRxs {
            shutdown: self.shutdown.subscribe(),
            broadcast_buffer_ch1: self.broadcast_buffer_ch1.subscribe(),
            broadcast_buffer_ch2: self.broadcast_buffer_ch2.subscribe(),
            broadcast_buffer_ch3: self.broadcast_buffer_ch3.subscribe(),
            broadcast_buffer_ch4: self.broadcast_buffer_ch4.subscribe(),
            broadcast_buffer_ch5: self.broadcast_buffer_ch5.subscribe(),
        }
    }
}

impl BroadcastRxs {
    pub fn resubscribe(&self) -> BroadcastRxs {
        BroadcastRxs {
            shutdown: self.shutdown.resubscribe(),
            broadcast_buffer_ch1: self.broadcast_buffer_ch1.resubscribe(),
            broadcast_buffer_ch2: self.broadcast_buffer_ch2.resubscribe(),
            broadcast_buffer_ch3: self.broadcast_buffer_ch3.resubscribe(),
            broadcast_buffer_ch4: self.broadcast_buffer_ch4.resubscribe(),
            broadcast_buffer_ch5: self.broadcast_buffer_ch5.resubscribe(),
        }
    }
}

/// ## Structen inneholder alle Broadcast kanalene
/// 
/// Navn på kanalene er matchende for `txs` og `rxs`:
///
/// | Variabel  | Beskrivelse  |
/// |-----------|-------------|
/// | **shutdown**  | Signaliserer til alle tråder at de skal avslutte |
/// | **update_task_status**  | Buffer til fremtidig bruk |
/// | **mpsc_buffer_ch2**  | Buffer til fremtidig bruk |
/// | **mpsc_buffer_ch3**  | Buffer til fremtidig bruk |
/// | **local_elev**  | Buffer til fremtidig bruk |
/// | **sent_tcp_container**  | Buffer til fremtidig bruk |
pub struct Broadcasts {
    pub txs: BroadcastTxs,
    pub rxs: BroadcastRxs,
}

impl Broadcasts {
    pub fn new() -> Self {
        let (shutdown_tx, shutdown_rx) = broadcast::channel(1);
        let (tx1, rx1) = broadcast::channel(1);
        let (tx2, rx2) = broadcast::channel(1);
        let (tx3, rx3) = broadcast::channel(1);
        let (tx4, rx4) = broadcast::channel(1);
        let (tx5, rx5) = broadcast::channel(1);

        Broadcasts {
            txs: BroadcastTxs {
                shutdown: shutdown_tx,
                broadcast_buffer_ch1: tx1,
                broadcast_buffer_ch2: tx2,
                broadcast_buffer_ch3: tx3,
                broadcast_buffer_ch4: tx4,
                broadcast_buffer_ch5: tx5,
            },
            rxs: BroadcastRxs {
                shutdown: shutdown_rx,
                broadcast_buffer_ch1: rx1,
                broadcast_buffer_ch2: rx2,
                broadcast_buffer_ch3: rx3,
                broadcast_buffer_ch4: rx4,
                broadcast_buffer_ch5: rx5,
            },
        }
    }

    pub fn subscribe(&self) -> BroadcastRxs {
        self.txs.subscribe()
    }
}

impl Clone for Broadcasts {
    fn clone(&self) -> Broadcasts {
        Broadcasts {
            txs: self.txs.clone(),
            rxs: self.rxs.resubscribe(),
        }
    }
}

// --- WATCH-KANALER ---
pub struct WatchTxs {
    pub wv: watch::Sender<Vec<u8>>,
    pub elev_task: watch::Sender<Vec<Task>>,
    pub watch_buffer_ch2: watch::Sender<bool>,
    pub watch_buffer_ch3: watch::Sender<bool>,
    pub watch_buffer_ch4: watch::Sender<bool>,
    pub watch_buffer_ch5: watch::Sender<bool>,
}

impl Clone for WatchTxs {
    fn clone(&self) -> WatchTxs {
        WatchTxs {
            wv: self.wv.clone(),
            elev_task: self.elev_task.clone(),
            watch_buffer_ch2: self.watch_buffer_ch2.clone(),
            watch_buffer_ch3: self.watch_buffer_ch3.clone(),
            watch_buffer_ch4: self.watch_buffer_ch4.clone(),
            watch_buffer_ch5: self.watch_buffer_ch5.clone(),
        }
    }
}

pub struct WatchRxs {
    pub wv: watch::Receiver<Vec<u8>>,
    pub elev_task: watch::Receiver<Vec<Task>>,
    pub watch_buffer_ch2: watch::Receiver<bool>,
    pub watch_buffer_ch3: watch::Receiver<bool>,
    pub watch_buffer_ch4: watch::Receiver<bool>,
    pub watch_buffer_ch5: watch::Receiver<bool>,
}

impl Clone for WatchRxs {
    fn clone(&self) -> WatchRxs {
        WatchRxs {
            wv: self.wv.clone(),
            elev_task: self.elev_task.clone(),
            watch_buffer_ch2: self.watch_buffer_ch2.clone(),
            watch_buffer_ch3: self.watch_buffer_ch3.clone(),
            watch_buffer_ch4: self.watch_buffer_ch4.clone(),
            watch_buffer_ch5: self.watch_buffer_ch5.clone(),
        }
    }
}

/// ## Structen inneholder alle Watch kanalene
/// 
/// Navn på kanalene er matchende for `txs` og `rxs`:
///
/// | Variabel  | Beskrivelse  |
/// |-----------|-------------|
/// | **wv**  | wv oppdateres av ´world_view_handler´ og leses av i ´get_wv´ |
/// | **update_task_status**  | Buffer til fremtidig bruk |
/// | **mpsc_buffer_ch2**  | Buffer til fremtidig bruk |
/// | **mpsc_buffer_ch3**  | Buffer til fremtidig bruk |
/// | **local_elev**  | Buffer til fremtidig bruk |
/// | **sent_tcp_container**  | Buffer til fremtidig bruk |
pub struct Watches {
    pub txs: WatchTxs,
    pub rxs: WatchRxs,
}

impl Clone for Watches {
    fn clone(&self) -> Watches {
        Watches {
            txs: self.txs.clone(),
            rxs: self.rxs.clone(),
        }
    }
}

impl Watches {
    pub fn new() -> Self {
        let (wv_tx, wv_rx) = watch::channel(Vec::<u8>::new());
        let (tx1, rx1) = watch::channel(Vec::new());
        let (tx2, rx2) = watch::channel(false);
        let (tx3, rx3) = watch::channel(false);
        let (tx4, rx4) = watch::channel(false);
        let (tx5, rx5) = watch::channel(false);

        Watches {
            txs: WatchTxs {
                wv: wv_tx,
                elev_task: tx1,
                watch_buffer_ch2: tx2,
                watch_buffer_ch3: tx3,
                watch_buffer_ch4: tx4,
                watch_buffer_ch5: tx5,
            },
            rxs: WatchRxs {
                wv: wv_rx,
                elev_task: rx1,
                watch_buffer_ch2: rx2,
                watch_buffer_ch3: rx3,
                watch_buffer_ch4: rx4,
                watch_buffer_ch5: rx5,
            },
        }
    }
}

// --- SEMAPHORE-KANALAR ---

pub struct Semaphores {
    pub tcp_sent: Arc<Semaphore>,
    pub sem_buffer: Arc<Semaphore>,
}

impl Semaphores {
    pub fn new() -> Self {
        Semaphores {
            tcp_sent: Arc::new(Semaphore::new(10)),
            sem_buffer: Arc::new(Semaphore::new(5)),
        }
    }
}

impl Clone for Semaphores {
    fn clone(&self) -> Semaphores {
        Semaphores {
            tcp_sent: self.tcp_sent.clone(),
            sem_buffer: self.sem_buffer.clone(),
        }
    }
}


// --- OVERKLASSE FOR ALLE KANALAR ---


/// ## Overklasse for alle interne kanaler
/// 
/// Inneholder `MPSC`, `Broadcast` og `Watch` kanaler
pub struct LocalChannels {
    pub mpscs: Mpscs,
    pub broadcasts: Broadcasts,
    pub watches: Watches,
    pub semaphores: Semaphores,
}

impl LocalChannels {
    pub fn new() -> Self {
        LocalChannels {
            mpscs: Mpscs::new(),
            broadcasts: Broadcasts::new(),
            watches: Watches::new(),
            semaphores: Semaphores::new(),
        }
    }

    pub fn subscribe_broadcast(&mut self) {
        self.broadcasts.rxs = self.broadcasts.subscribe();
    }

    pub fn resubscribe_broadcast(&mut self) {
        self.broadcasts.rxs = self.broadcasts.rxs.resubscribe();
    }
}

impl Clone for LocalChannels {
    fn clone(&self) -> LocalChannels {
        LocalChannels {
            mpscs: self.mpscs.clone(),
            broadcasts: self.broadcasts.clone(),
            watches: self.watches.clone(),
            semaphores: self.semaphores.clone(),
        }
    }
}
