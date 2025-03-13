use std::time::{Duration, SystemTime};


pub struct Timer{
    timer_end_time : SystemTime,
    timer_active : bool
}

impl Timer {

pub fn init(timer_end_time : SystemTime, timer_active : bool) -> Self {
    Self {
        timer_end_time,
        timer_active
    }
}

pub fn timer_start(&mut self, duration : Duration) {
    self.timer_end_time = SystemTime::now() + duration;
    self.timer_active = true;
}

pub fn timer_stop(&mut self) {
    self.timer_active = false;
}

pub fn timer_timed_out(&self) -> bool {   
    return self.timer_active && SystemTime::now() > self.timer_end_time;
}

}