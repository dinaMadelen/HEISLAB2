System overviev:
===================
We are employing a primary-backup/master-slave architecture where each machine consists of two independent processes. Each machine has a slave module dealing with elevator interfacing in addition to either a master or backup module. 

## Master 
Assgin calls to slave as well as sending them to the backup(s). Only assigns orders, and thus turns on lights, after orders have been confirmed by backups, thus ensuring service guarantee. 

## Backup 
A backup of the assigned calls in case master crashes along with a timer to transition to the master phase in the case of a crash.

## Slave  
Acts according to the calls assigned to it by the master. Main loop consists of a finite state machine for elevator action.

## How to run
On each machine call ./run.sh id portNumber
Where id is the elevator id (normally 0-2) and the port to interface with the elevator server.
You may have to call the elevator server with the same port. This is done with either ./SimElevatorServer --port portNumber or elevatorserver --port portNumber at sanntidsalen. 

## Extra note:
The run.sh file will compile the program and start it, in addition to restarting it on encountering exit code 42. These are planned panics by us calling os.Exit(42) leading to a program restart. 