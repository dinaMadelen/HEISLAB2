Elevator project: ELEVATORSYSTEM

Modules:

- constants contains constants used in the project.

- single-elevator module contains code to run a single elevator. 

- Network-go module contains code to run a peer2peer set up with UDP connections.

- assignment module contains code to assign (set order state = 1) hall requests by 
  using elevator data from all alive peers. 

- distribution module will contain code to distibute orders (make orders confirmed 
  by setting order states to 2) to all alive peers in the network. Not implementet 
  yet: should include cyclic counter logic. Want barrier before order is set as confirmed. 



 