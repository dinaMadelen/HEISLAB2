impl Elevator{
    pub fn add_to_queue(&mut self, floor: u8) {
        if !self.queue.contains(&floor) {
            self.queue.push(floor);
            self.sort_queue();
        }
        else{
            self.print_status();
        }
    }
    
    pub fn sort_queue(&self) -> Vec<u8> {
        let mut sorted_queue = self.queue.clone();
        let (mut non_negative, mut negative): (Vec<u8>, Vec<u8>) = sorted_queue
            .into_iter()
            .partition(|&x| x >= 0);
    
        non_negative.sort();
        negative.sort();
    
        // Non-negative numbers first, negative numbers last
        non_negative.extend(negative);

        let (mut infront, mut behind): (Vec<u8>, Vec<u8>) = non_negative
        .into_iter()
        .partition(|&x| x <= self.current_floor);

        infront.extend(behind);
        return infront;
    }

}