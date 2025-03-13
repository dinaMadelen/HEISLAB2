Design documentation 

A peer-to-peer system Is implemented in the code snapshot. There are five modules implemented at this point in the process, namely Elevator_io, Network, Single-Elevator, Hall arbitration and Worldview. Elevator_IO handles the inputs/outputs for the elevator and acts as the interface to the elevator hardware. Network handles the communication between the different elevators. Single-elevator handles the logic for what the elevator should do at a given time. Hallassigner outputs which hall requests the local elevator should serve. And the last module worldview that is used to ensure the necessary consistency in the network. 

These modules use channels to communicate necessary information which the modules in turn handle. Every module has its own run function which is used in the main.go file as separate go threads. Every run function is designed to run different function based on what it receives in its channel. 









