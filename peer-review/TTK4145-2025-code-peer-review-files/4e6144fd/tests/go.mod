module tests

require Network-Protocol v0.0.0
require Constants v0.0.0
require Driver-Elevator v0.0.0
require Driver-Elevio v0.0.0

replace Network-Protocol => ../shared_state/Network_Protocol
replace Constants => ../Constants/
replace Driver-Elevator => ./../elevator/
replace Driver-Elevio => ./../elevator/elevio

go 1.16