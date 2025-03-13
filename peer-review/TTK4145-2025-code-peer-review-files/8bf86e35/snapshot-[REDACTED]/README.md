Documentation

We have chosen to use a master-slave design. 
At start each elevator is assigned with an id. If no backup is detected, the elevator will become the master, if 
it detects a backup but no master it will become the backup. If the master and the backup is detected, it will become a slave. 
The master distributes the incoming calls, but the individual single elevator modules handles calls by themselves including handling motor control and sensorinput. 
we have tried to implement an algorithm to make the distribution of calls effecient, but the algorithm is not working quite as it should right now, but all calls are served. 
The modules of our project is the elevator logic which is the code for the single elevator, the elevator control, which is the master-slave functionality,
and lastly the network communication. 

