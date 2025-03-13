# Finite State Machine for Local Elevator 

## Overview
This module implements the **FSM** for controlling a local elevator in a distributed elevator system. It manages elevator states, processes new orders, updates floor indicators, and handles obstruction events.  

## Features 
- Determines when the elevator should stop at a floor  
- Chooses the appropriate travel direction based on pending orders  
- Simulates estimated time until an elevator picks up a request  
- Handles new orders and updates hall lights accordingly  
- Reacts to obstruction events by stopping and resetting orders if needed  

## Key Functions

### `ShouldStop(elev Elevator) bool`
Determines whether the elevator should stop at the current floor based on pending orders and movement direction.  

### `ChooseDirection(elev Elevator) int`
Decides the next direction of movement based on the elevator's previous direction and pending orders.  

### `TimeUntilPickup(elev Elevator, NewOrder Order) time.Duration`
Simulates the elevator’s journey and returns the estimated time required to reach a requested floor.  

### `Run(elev *Elevator, ...)`
Main FSM loop that:  
- Processes incoming orders and determines movement.  
- Opens/closes doors based on conditions.  
- Handles obstruction events.  
- Updates the elevator state and sends periodic heartbeat signals.  

## Obstruction Handling 
- If the elevator is obstructed while the door is open, a **timeout** starts.  
- If the obstruction persists beyond the timeout, active **hall orders are deleted**.  
- If the obstruction is removed before the timeout, the elevator resumes normal operation.  

## Timers Used  
- **DoorTimer**: Keeps track of door opening duration.  
- **ObstructionTimer**: Monitors persistent obstructions and clears hall orders if exceeded.  
- **HeartbeatTimer**: Periodically updates the elevator state to the system.