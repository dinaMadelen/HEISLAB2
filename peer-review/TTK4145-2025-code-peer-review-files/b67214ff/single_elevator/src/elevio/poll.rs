use crossbeam_channel as cbc;
use std::thread;
use std::time;

use super::elev;

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct CallButton {
    pub floor: u8,
    pub call: u8,
}

/// Goes through all (cab,up, down) buttons, and checks status. Notification about change from negative to positive is sent throgh channel in parameterlist.
pub fn call_buttons(elev: elev::ElevatorDriver, ch: cbc::Sender<CallButton>, period: time::Duration) {
    let mut prev = vec![[false; 3]; elev.num_floors.into()];
    loop {
        for f in 0..elev.num_floors {
            for c in 0..3 {
                let v = elev.call_button(f, c);
                if v && prev[f as usize][c as usize] != v {
                    ch.send(CallButton { floor: f, call: c }).unwrap();
                }
                prev[f as usize][c as usize] = v;
            }
        }
        thread::sleep(period)
    }
}

/// Goes through all floors, and checks status. Changes from previous iteration is informed about in channel in parameterlist.
pub fn floor_sensor(elev: elev::ElevatorDriver, ch: cbc::Sender<u8>, period: time::Duration) {
    let mut prev = u8::MAX;
    loop {
        if let Some(f) = elev.floor_sensor() {
            if f != prev {
                ch.send(f).unwrap();
                prev = f;
            }
        }
        thread::sleep(period)
    }
}

/// Checks stop-button status. Changes from previous iteration is informed about in channel in parameterlist.
pub fn stop_button(elev: elev::ElevatorDriver, ch: cbc::Sender<bool>, period: time::Duration) {
    let mut prev = false;
    loop {
        let v = elev.stop_button();
        if prev != v {
            ch.send(v).unwrap();
            prev = v;
        }
        thread::sleep(period)
    }
}

/// Checks obstruction-switch status. Changes from previous iteration is informed about in channel in parameterlist.
pub fn obstruction(elev: elev::ElevatorDriver, ch: cbc::Sender<bool>, period: time::Duration) {
    let mut prev = false;
    loop {
        let v = elev.obstruction();
        if prev != v {
            ch.send(v).unwrap();
            prev = v;
        }
        thread::sleep(period)
    }
}
