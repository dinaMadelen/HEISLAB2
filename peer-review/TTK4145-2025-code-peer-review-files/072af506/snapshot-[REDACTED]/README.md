Elevator software
===========================

We have use a peer-to-peer system with a master/slave architecure. We use UDP broadcasting.

(See attached figures to get a graphical representation of what is presented here)

**Our status**:

We have implemented necessary modules to be integrated together, but we have not yet integrated them. This means, although the controller and the elevator communicate together correctly, we have not managed interaction between multiple elevators. Nor have we tested the slave/master interaction and having elevators being assigned service orders outside of their node.
Note: the elevator and controller modules are complete, while the hall-order-assigner and network modules are not. For the network module, channel communication has been partly implemented. 
Logic for deciding primary/master and backup has not been implemented yet.
Take a look at usecase.md inside hall_order_assigner for more information about the assigner.

**Our project so far**:
The elevator software to run on a single computer is comprised of four main modules (outside the main function used to call them and run the program): main_elevator, main_controller, hall_order_assigner and network.

main_elevator and main_controller are part of the main package to allow for swift sharing of constants and flags. Variables are shared using message passing over channels.

Terminology to understand. In our architecture we distinguish between service orders and request orders. Service orders are orders supposed to be executed by an elevator (given by the master/primary). Request orders are requests registered from the button panel of the elevator, and sent up the chain to the controller and onwards to the master/primary. The request order is not added to the elevators queue. Eventually the request will be registered by the master/primary, and then be turned into a service order that one of the elevators will receive through its controller.

The main_elevator and main_controller modules are cooperative. The main_elevator controls the movement of the elevator and processes the I/O information from the button panel. This information is then passed on to the controller. The controller is sort of a local "administrator". It receives status updates from the elevator, and chooses actions to take based on this. The controller is also the interface between the elevator and the network.

Looking more closely at the controller, one will see that it is run as a state machine. Its behaviour will depend on the state.

When the elevator receives a new request order it will notify the controller. The controller will then *augment* the status of the elevator, by adding the request order to its status message (as if it was part of the elevator queue). This is used to reduce the number of message types, but the elevator is not adding this order to its queue yet. The master will then receive status messages from all controllers and create an overview of all active hall calls on the system. This is passed on to an assigner function, which reassigns all orders. It will then output an orderline containing all the orders and broadcast this to the network. Each controller will then extract its orderline (based on controller ID) and send these orders to its elevator to be added to the queue.

Looking at the network module, we see that there are two packages. When the controller is in any other mode than primary/master, it will be running the network_slave functionality. This allows the controller to send status messages to the network, and receive orderlines from the master. When the controller is in the primary mode, however, it will also compile the status messages from each elevator (which contains the request orders) and send this to the assigner. The assigner will return the service orders to be distributed through the master to the rest of the network. The master will run both network_master and network_slave, since its corresponding elevator should also receive orders.
