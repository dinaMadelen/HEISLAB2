//Some changes has been made to the original elvio:
    //Elevator has been renamed ElevatorDriver
    //CalllButton has been added a derive macro
    //Different types hass been assigned to the pub const-values
    
pub mod elevio {
    pub mod elev;
    pub mod poll;
}


pub mod execution{
    pub mod fsm;
    pub mod elevator;
    pub mod requests;
    pub mod timer;
    pub mod driver_communcation;
}

pub mod interface;

pub mod logic{
    pub mod controller;
}
