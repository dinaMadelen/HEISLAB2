# TTK4145---Project

## Architectural Overview

This project is an implementation of an elevator control system. The system is designed to handle multiple elevators in a building, ensuring efficient and safe transportation of passengers. The architecture is modular, with each module responsible for a specific aspect of the system.

## Note on Adapted Modules

The modules `hall_request_assigner`, `elevio`, and `network` are adapted from a published repository in the course GitHub. Ensure to review and understand these modules as they are integral to the system's functionality.

## Module Descriptions

### 1. Elevator Control Module (driver-go)
This module is responsible for the core logic of the elevator system. It handles the movement of the elevators, processing of requests, and coordination between multiple elevators.

### 2. Request Handling Module (cost_fns)
This module manages the requests made by passengers. It queues the requests, prioritizes them, and assigns them to the appropriate elevator.

### 3. Communication Module (network-go)
This module handles the communication between different parts of the system. It ensures that messages are correctly transmitted and received between the control module, request handling module, and the elevators.

### 4. User Interface Module (elevio inside driver-go)
This module provides the interface for passengers to interact with the elevator system. It includes the buttons inside the elevator and on each floor, as well as the display panels showing the status of the elevators.


