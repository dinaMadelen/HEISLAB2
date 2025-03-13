use serde::{Deserialize, Serialize};
use crate::interface::HallRequestMatrix;
use std::{collections::HashMap, process::Command};
use crate::logic::controller::ElevatorArgument;

//Note cost.rs are imported as a non-pub module in logic/controller.rs and public elements of this module are thus not pub to others than controller

/// Runs distribution executable and handles necessary json convertion of input and output
pub fn run_hall_request_assigner(input : HallRequestsStates,) -> Result<HallRequestsAssignments, String> {
    let input_json = serde_json::to_string(&input).unwrap();
    
    let output = Command::new("./hall_request_assigner")
    .arg("--input")
    .arg(&input_json)
    .output()
    .expect("Failed to start hall_request_assigner");

    if output.status.success() {
        Ok(serde_json::from_slice(&output.stdout).unwrap())
    } else {
        println!("Parsing error:  {:?}", output);
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

//======= Types to support convertion to correct json format for cost function executable input and output ========

pub type ElevatorArguments = HashMap<usize, ElevatorArgument>;
pub type HallRequestsAssignments = HashMap<usize, HallRequestMatrix>;

#[derive(Serialize, Deserialize)]
pub struct HallRequestsStates {
    #[serde(rename = "hallRequests")]
    pub hall_requests: HallRequestMatrix,
    pub states: ElevatorArguments,
}

