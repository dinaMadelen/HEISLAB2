<h1>FSM</h1>
<h2>config.go</h2>
<p>This file contains constants, message formats, as well as the ID's for the different roles.</p>
<h2>stateMachine.go</h2>
<p>This file is the pure logical functions for the singe elevator state machine. Contains the logic for computing state transitions, and deciding next moves. <br> All functions in this file is based on taking in information about the elevator at time t, and computing what to do at time t+1</p>
<h1>pba</h1>
<h2>backup.go</h2>
<p>This file was originally made to be the the program for the backup. It takes in the latest status message from the primary, and saves this. The status message contains information about the entire system, and the goal is that we can recover using the latest status. Unfortunatly this file has grown into containing code that is not executed by the backup. This means that restructuring, the part is neccessary as this i s contraintuitive in name. The backup also listens on the network for multiple messages from primaries, that dont have the same id. It merges the two primaries information, and demotes one of the primaries.</p>
<h2>prim.go</h2>
<p>This file was planned to contain the operation that a node would do if it was set to primary. This file sets up the neccessary channels. It monitors that it has connection to its backup, and can reelect a backup if this is lost. The primary sends status messages to its backup regularly. The primary also receives button presses, from the nodes on the network, and then it calls to distribute this orders to the most suitable node. After this is sends the confirmed order back to the node responsible for completing it.</p>

<h2>main.go</h2>
<p>This file is the main file of the program. It sets up the channels, and communicates. This file responsible for doing all interactions too hardware.</p>
<h2>timeIdle.go</h2>
<p>This file contains the logic for finding the elevator that should be given a order. It uses the qeue for the elevators, and finds the one that has the lowest estimated time to complete its queue. This elevator will be given the order.</p>