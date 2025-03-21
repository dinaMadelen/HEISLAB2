//Idea is that i make the elevator pass messages to the cab and the opposite
use crossbeam_channel as cbc;
use std::thread;
use cab::Cab;
use elevator_object::elevator_init::Elevator;

pub fn cab_elevator_info_transfer(cab: &Cab, status_rx: Receiver<Status>,queue_rx: Receiver<Vec<Order>>,req_status_tx: Sender<bool>,req_queue_tx: Sender<bool>){
    thread::spawn(move ||
        loop{
            cbc::select!{
                recv(status_rx) -> a => {
                    let status = a.unwrap();
                    
                },
                recv(queue_rx) -> a => {
                    let queue = a.unwrap();
                    
                },
                recv(req_queue_tx) -> a => {
                    let queue_request = a.unwrap();
                    
                },
                recv(req_status_tx) -> a => {
                    let status_request = a.unwrap();
                    
                },
            }
        }
    )
    
}