# ntnu_ttk4145_elevators

## Design
To tackle the Button Contract and to ensure fault tolerance to power outages or network connection loss, we design our system to be a peer-to-peer network. Each peer knows about the other elevator's states and the requests in the system. By keeping this information consistent enough among all peers, they can calculate which orders they will handle on their own. In case of a crash, they can regain the requested information from the other peers in the system so that no call is lost.

### The Requests and Order System
We define requests as follows: Each call button of the elevator (cab and hall) is associated with one request data object. The object contains information about its Origin (hall or cab + floor) and its current Status: Unknown, Absent, Unconfirmed, or Confirmed. Initially, the status of the request is Unknown. When no user wants to be picked up or dropped off at the Origin of a request its status is Absent. As soon as a user presses a cab or hall button the request with the corresponding Origin changes its status to Unconfirmed. The unconfirmed requests are distributed to all peers/elevators. When a single peer has received unconfirmed requests from all alive peers of the same Origin, the request state is changed to Confirmed. When a request has been handled by an elevator it changes the status back to Absent. This process is akin to a Cyclic Counter approach. The diagram below illustrates this FSM.
![RequestFSM](https://github.com/user-attachments/assets/60809c5d-57c3-4112-a610-89dde222c7f7)


One peer of the system shares the state of its local elevator and the requests it knows of with all other peers at a regular, fixed interval. If no update is received after a certain time is peer is considered as dead and will not be considered in the confirmation process of one request.

The request mechanism and regular sharing of the elevator states ensure that all peers have consistent enough information to convert requests into orders. We define an order as an instruction to the elevator to execute a request. Only confirmed requests are converted into orders. As every peer shares the same information when assigning requests, they all decide on a common order distribution.


## Structure
The project is divided into several modules which are separated by channels. Each module runs in its own routine. This enables a clean separation of responsibilities. The diagram below shows which modules exist and how they interact with each other.

![Modules](https://github.com/user-attachments/assets/86796711-9c2b-4447-bbf8-c36a1185ea02)
