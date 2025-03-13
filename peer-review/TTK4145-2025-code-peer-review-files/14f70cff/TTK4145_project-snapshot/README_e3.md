### Comments to the code
The project consists of the modules assignment, backup, distribution and elevator interface.

Assignment receives the elevator's Worldview consisting of all the states and orders for all elevators in the network, and calculates which elevator should take a new order based on the information.

The Backup module ensures that a new elevator spawns if the current crashes, managed by a read deadline.

Distribution is responsible for tracking elevators in the network, as well as UDP communication between the elevators.

Elevator interface tracks the cyclic counters for all orders.

The communication between most modules is yet to be implemented. This will be handled through channels.

Most of the distribution module is taken from the provided resources with slight changes. This also applies to fsm, timer and requests.
Elevio is fully taken from the resources.

The elevator struct is currently passed as a pointer from main to fsm and vice versa, this will be changed to a channel when the elevator_interface is fully implemented. 
