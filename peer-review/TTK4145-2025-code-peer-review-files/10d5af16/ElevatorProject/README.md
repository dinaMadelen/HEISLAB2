ElevatorProject
-----------------

Architecture:
- We have chosen a Master/slav architecture with a backup master. This is not currentlu fully implemented.

Protocol:
- We are using TCP

Modules:
- Driver: From project resources
- Communication: sets up communication between the master an slaves
- Elevator: Define the structs and functions relevant to an elevator object
- Elevator_io_device: Links the elevator object and the physical setup and its inputs
- Elevator_logic: defines the logic for the different roles
- FSM: finite state machine for an elevator object
- Resquest: handles requests based on the elevator object
- Timer: starts timer
- HRA: Hall request assigner. We are using the algorithm from the executable Project Resources
- Packet_loss: simulates packet loss, have not tried using it
