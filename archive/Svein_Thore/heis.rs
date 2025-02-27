

// Possible states for the elevator
enum Status{
    Idle,
    Moving,
    Maintenance,
    Error
}


// Elevator 
struct elevator{

    id:i8,
    current_floor:i8,
    going_up:bool,
    queue:Vec<i8>,
    status:Status
}


// Functions for elevator struct
impl elevator{

    // Add a floor to the queuem then sorts the queue. 
    fn add_to_queue(&mut self, floor: i8) {
        if !self.queue.contains(&floor) {
            self.queue.push(floor);
            self.queue = sort_queue(&self);
        }
        else{
            send_status();
        }
    }
    

    // Sets current status (Enum Status) for elevator,
    fn set_status((&mut self, Status:status)){

        match status{

            Status::Maintenance => {
                self.status = Status::Maintenance;
                self.queue.clear();
            }

            // Floors are read as i8, + is going up, - is going down.
            Status::Moving => {
                if self.queue.first() < self.current_floor {
                    self.direction = -1;
                    self.current_floor = self.queue.first();
                    self.queue.remove(0);
                } else{
                    self.direction = 1;
                    self.current_floor = self.queue.first();
                    self.queue.remove(0);
                }
            }

            Status::Idle => {
                self.status = Status::Idle;
                self.direction = 0;
            }

            Status::Error => {
                self.status = Status::Error;
                self.queue.clear();
                send_status();
            }
        }

    fn sort_queue(&self) -> Vec<i8> {
        let (mut non_negative, mut negative): (Vec<i8>, Vec<i8>) = self.queue
            .into_iter()
            .partition(|&x| x >= 0);
    
        non_negative.sort();
        negative.sort();
    
        // Non-negative numbers first, negative numbers last
        non_negative.extend(negative);

        let (mut infront, mut behind): (Vec<i8>, Vec<i8>) = non_negative
        .into_iter()
        .partition(|&x| x <= self.floor);

        infront.extend(behind);
        return infront;
    }

    // Moves to next floor, if empty queue, set status to idle.
    fn go_next_floor(&mut self) {
        if let Some(next_floor) = self.queue.first() {
            if *next_floor > self.current_floor {
                self.direction = Some(u8::1);
                self.current_floor += 1;
                set_status((&mut self, Moving));
            } else if *next_floor < self.current_floor {
                self.direction = Some(u8::-1);
                self.current_floor -= 1;
                set_status(&mut self, Moving);
            } else {
                self.direction = None;
            }
        }
        else {
            set_status((&mut self,Idle));
        }
    }
    }

    fn send_status(&self){


    }

    


}

main(){



}