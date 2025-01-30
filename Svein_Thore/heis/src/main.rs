

// Possible states for the elevator
enum Mode{
    Idle,        // Default state.
    Open_door,   // Can only be entered from idle and go to idle, timer based.
    Moving,      // Moving between floors.
    Maintenance, // Door open, bottom floor, stop prosessing orders, relenquish master role if master.
    Error_mode   // Stop processing orders, relenquish master role if master.
}

// Possible directions for the elevator
enum Direction{
    Up,
    Down,
    Idle
}

enum Role{
    Slave,
    Master,
    Inactive
}

// Elevator, containts all data relating to the current & planned upcomming tasks of an elevator.
struct Elevator{
    role:Role
    id:u8,                  //Decides hierarchy for master role
    current_floor:i8,       //Last sensor activated by elevator
    direction:Direction,    //Direction the elevator is headed
    queue:Vec<i8>,          //List of orders to be processed by this elevator
    mode:Mode               
}


// Functions for elevator struct
impl Elevator{

    // Add a floor to the queue, unless it is already in the queue. then sorts the queue. 
    fn add_to_queue(&mut self, floor: i8) {     
        if !self.queue.contains(&floor) {
            self.queue.push(floor);
            self.queue = sort_queue(&self);
        }
        self.send_status();
    }
    

    // Sets current mode (Enum Mode) for Elevator
    fn set_mode((&mut self, Mode:new_mode)){

        // Case switch for enum mode
        match new_mode{

            mew_mode::Maintenance => {
                self.mode = Mode::Maintenance;
                self.role = Role::Inactive;
                self.give_away_queue();
                self.queue.clear();
                self.send_status();
            }

            // Floors are read as i8, + is going up, - is going down.
            new_mode::Moving => {
                if self.queue.first() < self.current_floor {
                    self.direction = Direction::Down;
                    self.current_floor = self.queue.first();
                    self.queue.remove(0);
                } else{
                    self.direction = Direction::Up;
                    self.current_floor = self.queue.first();
                    self.queue.remove(0);
                }
                self.send_status();
            }

            new_mode::Idle => {
                self.mode = Mode::Idle;
                self.direction = Direction::Idle;
                self.send_status();
            }

            new_mode::Error => {
                self.mode = Mode::Error;
                self.role = Role::Inactive;
                self.direction = Direction::Idle;
                self.give_away_queue();
                self.queue.clear();
                self.send_status();
            }}

    // Sort queue       
    fn sort_queue(&self) -> Vec<i8> {
        let (mut positive, mut negative): (Vec<i8>, Vec<i8>) = self.queue
            .into_iter()
            .partition(|&x| x >= 0);
    
        positive.sort();
        negative.sort();
    
        // Non-negative numbers first, negative numbers last
        positive.extend(negative);

        let (mut infront, mut behind): (Vec<i8>, Vec<i8>) = positive
        .into_iter()
        .partition(|&x| x <= self.floor);

        infront.extend(behind);
        return infront;
    }

    fn update_floor((&mut self, floor:i8)){
    if self.direction = Up{
        self.floor = sensor;
    }
    else{
        self.floor = -1*sensor;
    }
    }

    // Moves to next floor, if empty queue, set status to idle.
    fn go_next_floor(&mut self) {
        if let Some(next_floor) = self.queue.first() {
            if *next_floor > self.current_floor {
                self.direction = Some(u8::1);
                self.current_floor += 1;
                set_status((&mut self, Mode::Moving));
            } else if *next_floor < self.current_floor {
                self.direction = Some(u8::-1);
                self.current_floor -= 1;
                set_status(&mut self, Mode::Moving);
            } else {
                self.direction = None;
            }
        } else {
            set_status((&mut self,Mode::Idle));
        }
    }


    fn send_status(&self){
        //TODO:Broadcast elevator struct
    }

    fn master_alive(){
        //TODO: Master broadcasts ID
    }

    fn master_check(&self,&message){
        //TODO: Check if currernt master has lower id and if true, take master role 
    }

    fn reset(&self){
        //TODO: Init, set direction, ID and check incomming broadcast from current master. if slave or master role based on ID.
        // go to a specific start point?, fint current floor?

    }

}

fn update_recived_data(&message){
    //TODO: Update elevator with new data
}

fn countdown_missing_master(){
    //TODO: Timer that checks every 100ms? if the master has sendt a broadcast, if not wait (100*id ms)? check again and then assume master role
}

main(){



}