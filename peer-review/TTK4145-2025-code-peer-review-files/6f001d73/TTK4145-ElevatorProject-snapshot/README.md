# TTK4145-ElevatorProject

This is the repo for the elevator project, group [REDACTED]

The project aims to implement an elevator control system in Go using a peer-to-peer system with UDP broadcasting for communication, which should be able to control 3 elevators over 4 floors in a reasonable way according to the given specification. The system consists of several modules which handles different functionalities of the system, ranging from communication between elevators to assigning hallorders to different elevators or turing on/off lights. Below is a desciption of each module:

## Modules

### **elevator**
Handles the core functionality of the elevator, including movement, door control, order handling, lamp handling, and finite state machine (FSM) logic.

### **elevio**
TTK4145 developed this network module, which you can access [here](https://github.com/TTK4145/driver-go). It provides an interface for interacting with the elevator hardware.
Although we have made minor adjustments to the code to fit our project, it remains largely unchanged.


### **assigner**
Assigns the hall requests to different elevators based on an algorithm using the provided example code in project resources which is accesible [here](https://github.com/TTK4145/Project-resources/tree/master/cost_fns). Should have a main function that takes the elevatorstates and hallrequests as input and outputs which request are assigned to which elevators.

### **commonstate**
Is far from finished, but should create a struct which keeps track of the state of the whole system (all elevators and orders), as well as making sure all elevators have a syncronized worldview. This also includes functions to manange the commonstate-struct. A final-state-machine should be implented in order to use this commonstate in a logical way, so that it is in fact syncronized.

### **network**
TTK4145 developed this network module, which you can access [here](https://github.com/TTK4145/Network-go). 
Although we have made some minor modifications, the module is mostly unchanged.

### **config**
Stores configuration parameters that several modules use, e.g. number of floors.