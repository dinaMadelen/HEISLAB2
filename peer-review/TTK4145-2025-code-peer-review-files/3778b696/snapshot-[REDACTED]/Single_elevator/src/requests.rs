use crate::elevator::{ButtonType, Direction, Elevator};

pub const N_FLOORS: usize = 4;
const N_BUTTONS: usize = 3;

pub struct Requests {
    requests: [[bool; N_BUTTONS]; N_FLOORS], // Matrise for alle knappetyper
    queue: Vec<i32>, // En kø for å holde styr på hvilke etasjer som har forespørsler
}

impl Requests {
    pub fn new() -> Self {
        Requests {
            requests: [[false; N_BUTTONS]; N_FLOORS],
            queue: Vec::new(),
        }
    }

    pub fn add_request(&mut self, floor: i32, button: ButtonType, elevator: &Elevator) {
        let button_idx = button.to_u8() as usize;
        self.requests[floor as usize][button_idx] = true;
        println!("Added request: Floor {}, Button {:?}", floor, button);
        
        if !self.queue.contains(&floor) {
            self.queue.push(floor);
            self.sort_requests(elevator);
        }
        self.print_queue();
    }

    pub fn should_stop(&self, elevator: &Elevator) -> bool {
        let floor = elevator.floor as usize;
    
        let cab_call = self.requests[floor][ButtonType::Cab.to_u8() as usize];
        let hall_up = self.requests[floor][ButtonType::HallUp.to_u8() as usize];
        let hall_down = self.requests[floor][ButtonType::HallDown.to_u8() as usize];
    
        match elevator.dirn {
            Direction::Up => cab_call || hall_up || (hall_down && self.is_last_up_stop(elevator.floor)),
            Direction::Down => cab_call || hall_down || (hall_up && self.is_last_down_stop(elevator.floor)),
            Direction::Stop => cab_call || hall_up || hall_down,
        }
    }
    
    pub fn is_last_up_stop(&self, floor: i32) -> bool {
        self.queue.iter().all(|&f| f <= floor)
    }
    
    pub fn is_last_down_stop(&self, floor: i32) -> bool {
        self.queue.iter().all(|&f| f >= floor)
    }
    

    pub fn clear_request(&mut self, floor: i32) {
        for btn in 0..N_BUTTONS {
            self.requests[floor as usize][btn] = false;
        }
        self.queue.retain(|&f| f != floor);
        println!("Cleared request for floor {}", floor);
    }

    pub fn choose_next_floor(&self) -> Option<i32> {
        self.queue.first().copied()
    }

    pub fn sort_requests(&mut self, elevator: &Elevator) {
        if self.queue.is_empty() {
            return;
        }

        let current_floor = elevator.floor;
        let direction = elevator.dirn;

        self.queue.sort_by(|a, b| match direction {
            Direction::Up => {
                if *a >= current_floor && *b >= current_floor {
                    a.cmp(b)
                } else if *a < current_floor && *b < current_floor {
                    b.cmp(a)
                } else if *a >= current_floor {
                    std::cmp::Ordering::Less
                } else {
                    std::cmp::Ordering::Greater
                }
            }

            Direction::Down => {
                if *a <= current_floor && *b <= current_floor {
                    b.cmp(a)
                } else if *a > current_floor && *b > current_floor {
                    a.cmp(b)
                } else if *a <= current_floor {
                    std::cmp::Ordering::Less
                } else {
                    std::cmp::Ordering::Greater
                }
            }

            _ => a.cmp(b),

           
        });
    }

    pub fn print_queue(&self) {
        if self.queue.is_empty() {
            println!("Queue is empty.");
        } else {
            println!("Queue: {:?}", self.queue);
        }
    }
    

    pub fn has_request_at(&self, floor: i32) -> bool {
        self.requests[floor as usize].iter().any(|&x| x)
    }

    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }
}
