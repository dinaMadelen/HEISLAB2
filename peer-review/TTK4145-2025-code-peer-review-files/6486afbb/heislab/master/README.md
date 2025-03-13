Master module
===================

Consists of the following files:
- assigner.go
- backupComm.go
- lookFor.go
- master.go
- slaveComm.go
- worldview.go

## assigner.go
Contains a routine and functions to assign orders upon receiving new confirmed orders or new elevator states.
It assigns orders using the given hall_request_assigner before sending them to the slaves.

## backupComm.go
Contains a go routine to communicate with the backups and get acknowledgments on updated states which is then sent to the assigner for assignments. Also deals with the case of multiple masters. If there is another master, it will either incorporate it's orders into it's own if it has a lower id, or crash after having confirmed the other master has gotten it's own calls. 

## lookFor.go
Contains a go routine that continously listens for other masters and sends their orders and ID to the backupComm routine if it finds another master.

## master.go
Contains a function that initializes all necessary channels and starts all necessary go routines. 

## slaveComm.go
Contains two main parts:
- A part that continously broadcasts the most recent assignments over UDP to the slaves
- A part for receiving event messages and sending acknowledgements to the slaves.
    - This part sends acknowledgments on each incoming message, and translates new messages to either state updates or order updates. Order updates are sent to the backup communicator and state updates are sent to the assigner.

## worldview.go
Contains structs to track the worldview of the master as well as formating calls. 
