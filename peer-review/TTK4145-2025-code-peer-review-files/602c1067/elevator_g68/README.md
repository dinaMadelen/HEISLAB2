# elevator_g68

## Implementation Plan

### Alternative solution
```
module ElevatorController
    + Run
        - start peerState manager
        assign order when incoming button press

module PeerStateManager:
    - list of currently alive elevators and their state

    + broadcastHeartbeatThread
        - serializes <nonce, elevator_id, floor, direction, requests>
        - maybe state
        - maybe use protobuff
        - sends in continuous interval

    + receiveHeardbeatThread
        - Updates elevator state
        - if no message received for an elvator => mark dead
            - pick a random delay for starting computation
            - if we received a new winner for this round before then => we do not recompute
            - TODO take over orders

    + getElevatorState(id)
        - returns current state of the elevator


module RequstScheduler:
    + assignOrder
        - called when button press registered
        - calculates cost
        - broadcast winner 
            - rebroadcast if state still unchanged after X sec

    + receiveOrderThread
        - listens for broadcast cost winner messages
        - if ID matches my ID => 
            - updates its state
            - sends out heartbeat
            - persists to file for restart
            - lights up confirmation button

```

- order receiver knows state of all other nodes
- computes costs for each
- assign order to cheapes elevator
    => broadcast <hallcall, floor, direction, elevator id>
- once responssiblle elevator recieves, it updates it's state, lights up confirmation button
    => other elevators will know of elevators responsiblity through heartbeat => if none receives heartbeat, we also persisted the order => so worst case when it comes back alive it will take the order


- heartbeat monitor dispatches new elevator to take over if an elevator failed
=> here potentially multiple elevators could get dispatched => how can we have just one leader?
    - all elevators in same partition should end up with the same solution for the assignee => if not we just dispatch multiple elevators

### Idea: Broadcast costs for a hall call

interface RequestHandlingStrategy:

class RequestHandlingStrategy:
    - requestMatrix => each button has the ID of the elevator that was assigned to it; -1 for not assigned

    + MotorDirection getNextDirection(request matrix/internal request DS)
    + bool stopOnFloor(request matrix, floor, movement direction)
    + addRequest() => persists in internal DS
    + clearRequest(floor, button type) => removes request
    - clearRequests_ => function pointer with/without side effect
    + costOfRequest()
        . copy current elevator state - including request table
        . stopOnFloor -> clear requests and add door open time
        . choose new direction & add a constant travel time for between floors
        . go until we are in state ST_Idle = all requests cleared
        - ide elevators must choose a direction
        - elevators with door open must pretend door is closed


class AssignmentModule:
    - map (nonce & floor & button) => list of (elevator_id & cost)

    + broadcastCost(elevator_id, cost, button, floor, nonce)
        - main calculates cost -> calls this
        - nonce:
            - set by button press receiver, forwarded by broadcast receivers
            - goal: disambiguate voting rounds -> has to be unique across rounds (similar to ID)

    + registerCost(elevator_id, cost, button, floor, nonce)
        - add to map
        - if map full && you are lowest_cost or lowest ID => dispatch
        - if map full && you are not lowest_cost or lowest ID => mark ID of elevator in table

    + elevatorBecameUnavialiblle(elevator_id)
        - alternative -> use livenessModule to check if each elevator is alive...
        - remove requirement for this elevator
        - either in list of allive nodes or in map...

    + RetransmittionRequesterThread
        - for each missing cost -> retransmit my own cost message to this elevator (send only to its address)
        - run once every N ms
        - only request for alive elevators

    + broadcastPickup/OrderFilled
        - how can i know an elevator has filled it's order?
            - it did not go offline in x timeunits
            - it heartbeats the open hall-calls `(bool, bool) * floors` => 8 byte message
            - it informs of each fill => how to have delivery certainty though?

        - called when an elevator filled an order
        => TODO is this even necessary?
            - elevator died after pickup but nobody received message => second pickup dispatched
            - elevator remains alive -> other elevator might think this floor is already covered by this elevator...


TODO reconfirm this => think especially about nonces and validity of old orders & state misamtches
think about request table state -> when do we have to be consistent?


class LivenessModule:
    + broadcastHEartbeatThread
    + receiveHeartbeatsThread

    - set of alive elevators


class ComesBackAlive:
    + sendRequest to other nodes to get their state



# TODO
- motor failure => press stop 8 continuously
- persist hall calls to make sure wo never lose them







=> incoming buttonPress: calculate cost and broadcast
<elevator_id, cost, floor, direction>

in main loop on receive of cost message:
- calculate own cost and broadcast
- add cost to DS
- start timer - after this time request costs again from all missing nodes
- if no requests missing => lowest cost node sets button light
- adds request to it's orders

- send request cleared message to all nodes once we are at a floor for pickup
    - how can we be sure received by all => we don't have to be


2. Failure Mode (node that's responsible for pickups dies)
- Threads that send and receive heartbeats
- notify elevator via channel of offline elevators
- restart cost calculation -> How to restart? Sequence number?


=> table filled with IDs of nodes responsible for an order


3. Restart after failure
- request table from another node
- set lights accordingly