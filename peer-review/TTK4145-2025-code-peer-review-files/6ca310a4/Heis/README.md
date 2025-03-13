
The elevator system is supposed to use UDP in a peer to peer nettwork. We have implemented three "main" modules

- fsm
- orders
- network

The fsm module works as a standalone elevator. Button inputs are passed to the order module and orders for the elevator are sent back to the fsm.
It handles initialization, active requests, opening and closing of the door and when arriving on a floor. 
This module also keeps track of its own active orders in a bool 2d array. When an order is given by the order module true is written in the array, and when an order is cleared/set to false a message of this is sent to the order module. All button inputs are passed forward to the order module

The order module is responsible for handling all orders, either comming from a local button press or from updates on the network.
All elevators on the network keeps track of the other elevators orders in a map called AssignedOrders, where the keys are elevator id's
and the values are a 2d slice of assigned orders for the corresponding elevator implemented as a cyclic counter. 
The module is responsible for synchronization of orders, assigning orders from a button press to the correct elevator and reassigning uncompleted orders. When an order should be done by the local elevator this is sent as a message to the fsm module, and when an order is cleared in the fsm the order module updates the AssignedOrders map for that elevator. 

The network module is responsible for sending information to the other elevators using UDP. It sends the elevator states and id many times a second which makes it able to keep track of elevators disconnecting and connecting assuming an elevator is disconeccted when it does not send its id. This should also handle packet loss. The elevator states is JSON enoded.

In addition there are also som minor "help" modules/files

- config: Loads parameters from config file.
- elevator: Types and structs correspondind to an elevator and its behaviour for both fsm and order modules.
- msgTypes: Types of messages being sent between different elevators.
- timer: general timer functionality. Timer length is given to module as input.