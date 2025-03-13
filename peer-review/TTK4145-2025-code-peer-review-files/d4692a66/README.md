

## Project Structure

```
go.mod
Heis/
    communication/
        heartbeat.go
        StateUpdate.go
    config/
        config.go
    driver/
        elevator-server-master/
            LICENSE
            README.md
            src/
                c/
                    basic_driver/
                        arduino_io_card.c
                        arduino_io_card.h
                    elevio_driver/
                        elevio.c
                        elevio.h
                d/
                    arduino_io_card.d
                    elevatorserver.d
    elevio/
        elevator_io.go
    FSM/
        FSM.go
    logic/
        elevatorcontrol.go
        hall_request_assigner.go
        taskmanager.go
    network/
        bcast/
            bcast.go
        conn/
            bcast_conn_darwin.go
            bcast_conn_linux.go
            bcast_conn_windows.go
        localip/
            localip.go
        peers/
            peers.go
main.go
```

## Getting Started

### Prerequisites

- Go 1.16 or later
- A D compiler (for building the elevator server)
- A C compiler (for building the low-level driver)

### Building the Project

1. Clone the repository:

```sh
git clone https://github.com/yourusername/elevator-control-system.git
cd elevator-control-system
```

2. Build the elevator server:

```sh
cd Heis/driver/elevator-server-master/src/d
dmd arduino_io_card.d elevatorserver.d -ofelevatorserver
```

3. Build the low-level C driver (optional):

```sh
cd Heis/driver/elevator-server-master/src/c/basic_driver
gcc -o arduino_io_card arduino_io_card.c
```

### Running the Project

1. Start the elevator server:

```sh
./elevatorserver
```

2. Run the main Go application:

```sh
go run main.go
```

### Configuration

The configuration for the elevator system is located in the config.go file. You can set the number of floors, buttons, and other parameters as needed.

## Project Components

### Main Components

- main.go: The entry point of the application. Initializes the elevator system and starts the main control loop.
- config.go: Contains configuration settings and data structures for the elevator system.
- elevator_io.go: Handles communication with the elevator hardware.
- FSM.go: Implements the finite state machine for controlling the elevator.
- elevatorcontrol.go: Contains logic for controlling the elevator, including handling requests and updating button lights.
- hall_request_assigner.go: Assigns hall requests to elevators.
- taskmanager.go: Manages tasks and orders for the elevators.

### Communication

- heartbeat.go: Handles heartbeat messages between elevators to detect online/offline status.
- StateUpdate.go: Manages state updates and broadcasts them to other elevators.

### Network

- bcast.go: Implements broadcasting of messages over the network.
- conn: Contains platform-specific implementations for network connections.
- localip.go: Retrieves the local IP address.
- peers.go: Manages peer updates and detects new/lost peers.







