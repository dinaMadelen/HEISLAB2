/*
use std::time::{Duration, Instant};
use std::sync::Mutex;

lazy_static::lazy_static! {
    static ref TIMER: Mutex<Timer> = Mutex::new(Timer::new());
}

struct Timer {
    end_time: Option<Instant>,
    active: bool,
}

impl Timer {
    pub fn new() -> Self {
        Timer {
            end_time: None,
            active: false,
        }
    }

    pub fn start(&mut self, duration: f64) {
        self.end_time = Some(Instant::now() + Duration::from_secs_f64(duration));
        self.active = true;
    }

    pub fn stop(&mut self) {
        self.active = false;
    }

    pub fn timed_out(&self) -> bool {
        self.active && self.end_time.map_or(false, |end| Instant::now() > end)
    }
}

pub fn timer_start(duration: f64) {
    let mut timer = TIMER.lock().unwrap();
    timer.start(duration);
}

pub fn timer_stop() {
    let mut timer = TIMER.lock().unwrap();
    timer.stop();
}

pub fn timer_timed_out() -> bool {
    let timer = TIMER.lock().unwrap();
    timer.timed_out()
}
*/
