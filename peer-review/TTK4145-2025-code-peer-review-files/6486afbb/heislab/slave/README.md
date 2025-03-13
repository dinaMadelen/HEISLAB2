Slave module
===================

Consists of the following files:
- elevator.go
- fsm.go
- io.go
- network.go
- requests.go
- slave.go
- timer.go

## elevator.go
Contains the elevator struct which tracks the state of the elevator. 
Also contains a function to check the validity of an elevators state and a function to update the current elevator object along with interfacing with IO and sending elevator state change messages.

The elevator_updateElevator function is always called after an fsm function. 

## fsm.go
Contains all the functions for the fsm as well as the main loop that activates them and keeps track of the elevator state. The functions are called when the elevator detects the corresponding event for each fsm function. These are as follows:
- Initialization
- Updated requests
- The elevator has arrived at a floor
- There has been a change in the door obstruction switch
- The stop button has been pressed
- The elevator timer has timed out

## io.go
Contains a function to activate the IO of the elevator based on its current state. This is called by the elevator_updateElevator function.
Also contains a function to turn on or off the order lights based on an incoming lights update from the master. 

## network.go
Contains two functions run as go routines. One for sending messages to the master and one for receiving from. 
The sending function sends events and waits for an acknowledgement from the master before attempting to resend the same event.

The receiving function listens to order assignment UDP broadcasts from the master and translates them to updated order assignments and lights.

## slave.go
Contains a single function that initializes all channels and go routines between other parts of the slave module

## timer.go
Contains a timer server that will send a timeout on the t_end channel after the specified number of seconds sent on the t_start channel