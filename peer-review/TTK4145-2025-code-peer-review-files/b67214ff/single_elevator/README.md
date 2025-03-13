# General overview:  

Our project is divided into three general modules, the elevator, the controller and the interfacing. We are using the Master/Slave method, with little distinction between master and slave besides who is allowed to send orders. All units that are not masters are slaves. 

The elevator takes in a hall_request matrix and moves the single elevator accordingly. It also sends new hall_requests and new states to the controller. 

The controller holds an overview of hall requests and the states of each elevator. It then assigns hall_requests to the different elevators. It also keeps track of which elevators are connected to the system.  

The network interface deals with communication between the elevator module and controller module within each pair, but also with communication between the different elevators. Communication between elevator and controller is done through channels, and communication between different elevators is done with UDP broadcasting.  

## The elevator module can again be divided into two modules.  

An FSM that switches between the states “on request matrix”, “on floor arrival”, and “on door timeout”. On request matrix handles new requests from either the cab or the controller. On floor arrival handles arriving at a floor and completing an order. On door timeout handles closing the door (if there are no obstructions) and moving towards the next order after the door is closed. The fsm also has a “main” loop called run_fsm which switches between the states and also handles communication to and from the interface. 

A driver communication module that starts the elevator_driver and sets up polling which it uses to communicate with the FSM module through channels. 

The module is based on the single elevator from exercises 3. 

## The interface module
creates and distributes channels for communication. To do so, all other code is initiated from this module with proper channels as parameters. All communication goes through the functions broadcast, and all communication to and from separate pairs goes through receive. In the broadcast functions communication that goes within a pair are passed through on another channel directly to the controller and are not sent over UDP. Both broadcast and receive use only one socket each, and different messages are separated by their datatype defined in interface.rs. It is also in the broadcast the distinction between master and slave is made, a hierarchy position is assigned, 0 indicating the master. For each elevator, the number equals the number of connected elevators with lower (hard coded at initialization) elevator number. 

## The Controller Module
 mainly holds the data needed to properly run the handout executable correctly. It also includes a state machine which keeps track of the connectivity state of the controllers which will be used to support fault tolerance. The functions in the logic/controller.rs can be considered helpers. The program flow of the controller is in interface/controller.rs. The executable is run and orders sent out whenever the order matrix or connectivity state of any elevator changes. When disconnected, the hierarchy position is set to 0, the list of active elevators set to only oneself and all orders are set to zero. Then the controller and elevator should function as a single elevator without other modifications.  

## Current status
 is that order distribution seems too slow. Or communication jammed. A function to notify the controller to clear orders are not yet implemented, but the channels are put up to support it. Due to this we have not yet made any rigorous testing of the different parts of the system. We were not confident about the separation of messages by type, but it has been tested and seems valid. Also, we have not made support for packet loss handling or correct order of messages. 

## Recommendation to quickly acquaintance yourselves with our code.  

    If you want to run it, find appropriate terminal commands in main. Also, we remove the target file, so you have to run cargo build first.

    All communication goes through network_unit.rs, their names are structured in a logical manner/pattern (ETC = Elevator To Controller). All other code is initiated from this file. 

    Interface.rs holds datatypes used to separate between messages and each are linked to those of channels. 

    Cost.rs are weirdly included in logic/controller.rs, the same applies connectivity_sm into interface/controller.rs 

    We have included elevio  because we have made some minor changes to the original code. 

    The connectivity state machine has been implemented in a rustly manner and looks very different from a typically c state machine. Try to ignore the technicalities and focus on the trigger->transition logic.  

    