# HEISLAB2
Heislab 2 

FULL IDEA:
Make a master-slave system where the master is chosen based on lowest id. Acs and nacs are implementet through udp, and in case of death of the master/slave (No reply for a certain period of time) the master/ will be reelected/reimplemented in the same manner as on initialization, and new queues will be distributed.

Basic event of hall button press:
- Slave recieves button press
- Slave broadcasts button press
- Master calculates correct floor
- Master sends out request to elevator that should take it
- Slaves should ack, if not master takes order
- All elevators updates the queues and active orders of known elevators

Every message contains ID, which makes all elevators aware of who is supposed to be Master. To ensure safety all messages are encoded with a hash that is calculated based on the message. The system will reject messages that are not corrected properly. There is also a filter that makes sure the sender is coming from the same same subnet.

The system continously monitors for dead elevators, and master complications. This makes sure the system always distributes dead elevator orders, and in case of emergencies and no broadcasting from other elevators the system will set itself to master and overtake all calls exept CAB calls from other systems.



Threads that will be active:
    Main thread: This controls general function and how to act in accordance with the buttonpresses and inputs from the physical elevator.

    Master monitoring thread: Makes sure there are no complications with the system masters.

    Queue finisher thread: Pings the elevator to go_to_next_floor to make sure the elevator always has correct light function and forces the elevator to always try to move in case the motor is disconnected and orders redistributed, so it can broadcast that it is alive in case of "reawakening". 

    Reciever: This thread spawns new threads when a message is recieved, it makes the system always able to recieve messages and spawns new handlers to deal with the messages. This is the "main" communications hub.

Mode of communication:
    The project is mainly written with mutexes and a tiny bit of message passing. This is something we slightly regret. 

State of the code: 
    The code is not finished and not that organized. But it is on its way there... sooon..... maybe....





