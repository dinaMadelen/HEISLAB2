# TTK4145_Project

1. Group [REDACTED]
2. Lab station 2
3. Members
    1. ntnu_usrname: hastrad,  ntnu_mail: hastrand@stud.ntnu.no
    2. ntnu_usrname: jonaskgu, ntnu_mail: jonaskgu@stud.ntnu.no
    3. ntnu_usrname: andrwr,   ntnu_mail: andrwr@stud.ntnu.no
4. [TA-assistance](https://forms.office.com/pages/responsepage.aspx?id=cgahCS-CZ0SluluzdZZ8BTRdfsi9Ri5Enu0trVtR9BxUQzJINzVHVTBWMk1BWjBOWjgyRUJSSDVEMC4u&route=shorturl)

## Documents
Some project documentation and information can be found in teh Project documentation folder.

## Strategy

Our selected stategy is to design a pure P2P network (all nodes are equal).
The strategy exploits the fact that the provided order assignment algorithm is deterministic; if all nodes agree on the system state, they will arrive at the same order assignment. There is no need for a master node to coordinate the elevators.
The key design challenge is to ensure that the nodes agree on the system state. That is, maximize the degree of consistency in the network under the constraint of guaranteed Avalability and Partition Tolerance (as described by the CAP theorem).

To ensure a high degree of consistency, the following information sharing is selected:

- The system is modelled as a Finite State Machine (FSM) with numbered states.
- A state in the FSM includes all data necessary to produce output (order assignment). Incidentally, the system state is (mostly) contained in the input to the HallRequestAssigner algorithm.
- Each node contains an instance of FSM, which represents that node's worldview.

Worldview: State of all peers, order matrix, and state ID. Essentially, a message is a copy of the local data of a node.

- State transitions happen when messages are passed between nodes. The node sending the message is the only source of truth, and its worldview is accepted by all receiving nodes.
- Messages are filtered (discarded) based on state number. Nodes only accept messages with state number higher than its own worldview (special handling of wrap-around, cyclical counter). This is to ensure commutativity and idempotency.
- When we do need negation (such as resetting the cyclical counter), we ensure synchronization between nodes using barriers. Generally, we avoid negation whenever possible.

WE MAY CHOOSE TO USE A MONOTONICALLY INCREASING COUNTER; INCREMENTING A UINT64 AT 10HZ PRODUCES OVERFLOW IN ~6*10^11 YEARS...

## Resources  

[Testing at home](https://github.com/TTK4145/Project/blob/140c9d37b9b30d9daf76ce7b543396791eac4c3d/testing_from_home.md)
[Network-go](https://github.com/TTK4145/Network-go)  
[Project resources](https://github.com/TTK4145/Project-resources/tree/master)  


## Project deadlines

| What                                | Description                                              | Date                 |
| :---                                |    :----:                                                |         :---:        |  
| Exercise 1                          | Intro Golang and C                                       | 17.01.2025           |
| Exercise 2                          | Intro TCP and UDP                                        | 17.01.2025           |
| Preliminary design description (1)  | Concise description of the design chosen for the project | 27.01.2025           |
| Exercise 4                          | Process pairs                                            | 03.02.2025           |
| Code snapshot handin                |  -----                                                   | 07.03.2025           |
| Code Handin                    (3)  |  -----                                                   | 28.03.2025           |
| Exercise 5                          | Synchronization                                          | xx.03.2025           |
| FAT test                       (2)  |  -----                                                   | 04.04.2025           |
| Exercise 6                          | Scheduling                                               | 21.04.2025           |
| Project Report                 (4)  | Handin final report                                      | Week 18 (28.04-04.05) |



## Project and Exercise Information
There are six exercises in the course, all of which must be approved by the student assistants at the lab. None of the exercises count toward the final grade but are meant to assist throughout the project. The project counts for 45% of the final grade (the exam makes up the last 55%).

**The evaluation of the project is split into four parts:**

1. Preliminary Design Description (5% ) (1)
2. Factory Acceptance Test (FAT)  (10%) (2)
3. Code quality                   (15%) (3)                
4. Final report                   (15%) (3)  

## Structure of project
Following the standard used in [go layout](https://go.dev/doc/modules/layout) (to be added as file tree).

## Additional content
- [Cost function](https://github.com/TTK4145/Project-resources/tree/master/cost_fns)
