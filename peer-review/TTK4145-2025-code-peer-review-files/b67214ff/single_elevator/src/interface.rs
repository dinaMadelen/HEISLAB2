pub mod network_unit;
pub mod elevator;
mod controller;

use crate::elevio::poll::CallButton;
use crate::execution::elevator::N_FLOORS;
use crate::logic::controller::ElevatorArgument;

//This file contains datatypes to support sending and receiving over UDP. To allow all communication to go over one socket, the datatypes are
//used to seperate between messages. I refer to the network_unit::Recieve functions loop to get an understanding of how it is used. Types are divided into
//three categories, ETC, CTC and CTE, denoting Elevator To Controller etc. Note also that this method will fail if the same datatype are included twice
//in a category. It is solvable, but one have to be cautious about it.


//============Defining datatypes for UDP messages CTE, CTC and ETC===================

pub type HallRequestMatrix = [[bool; 2]; N_FLOORS];

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct MessageDataType<R>{
    pub data: R,
    pub elevator_number:u8,
}

pub type HallRequestDataType = MessageDataType<HallRequestMatrix>;
pub type ElevatorArgumentDataType = MessageDataType<ElevatorArgument>;
pub type OrderStateDataType = MessageDataType<(CallButton, bool)>;
pub type InactiveStateDataType = MessageDataType<bool>;

#[derive(serde::Serialize, serde::Deserialize, Debug)]
#[serde(tag = "cte")] // Adds a "type" field in JSON
enum CTEMessageWrapper {
    HallRequest(HallRequestDataType),
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
#[serde(tag = "etc")] // Adds a "type" field in JSON
enum ETCMessageWrapper{
    //HallRequest(HallRequestDataType),
    ElevatorArgument(ElevatorArgumentDataType),
    OrderState(OrderStateDataType)

}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
#[serde(tag = "ctc")] // Adds a "type" field in JSON
enum CTCMessageWrapper {
    HallRequest(HallRequestDataType),
    InactiveState(InactiveStateDataType),
    ElevatorArgument(ElevatorArgumentDataType)
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
#[serde(tag = "msg_type")] 
enum MessageWrapper{
    CTE(CTEMessageWrapper),
    ETC(ETCMessageWrapper),
    CTC(CTCMessageWrapper)
}
