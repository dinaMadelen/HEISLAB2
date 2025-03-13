# TTK4145 Group [REDACTED]
### Project Description 
The program is a pure peer-to-peer, UDP-based system that utilizes a mesh network.

The system consist of four independent modules. For detailed relationship see the class diagram.

Driver: Manages all elevator sensors (floor and obstruction) and calculates the elevator’s current state. It propagates this state to the Assigner module and sets the motor direction as instructed by the Assigner. 

Assigner: Assigns confirmed orders received from the Network module and forwards these to the Lights and Driver modules. It also forwards new and completed orders to the Network module, as well as an updated elevator state.

Lights: Handles all button, floor and door lights based on input from the Assigner and Driver modules.

Network: Maintains the state of all elevators, propagates the cyclic counter, and transmits the local elevator state across the network. 

![Demo Image](images/ClassDiagram.png)

### Pre-requisites
* [hall_request_assigner (HRA)](https://github.com/TTK4145/Project-resources/releases/tag/v1.1.1) (v1.1.1) by [@klasbo](https://github.com/klasbo)

### Installation

Download the source repository as zip, and extract in desired directory.

Navigate into the project directory

```bash
cd <yourpath>/Project
```

Add HRA dependency to the `orderassigner` module

```bash
mv ~/Downloads/hall_request_assigner ./orderassigner/
```

### Build and Run
*Note: The ID must be in the range 0 to NElevators-1.*
```bash
chmod +x run.sh
./run.sh <ID> 
```
### Terminate Terminal

```bash
pkill -f run.sh
```