## Code explanation.

So far we have gotten the single elevator to work and set up as we want it. 
We have also done communcation between computers and been able to send the elevator object between computers. 
Further we have delved in to the others parts of the project and built understanding of it, but not coded it yet.

The elevator code is in go and is seperated into several files. These files make it easier to use the code while programming. All of the code is in the same package for simplicity. 
The code consists of three divisions: the state machine(elevator_state.go), the request handling (requests.go) and door handling timer (timer.go). 
The code should be using the standard convention from lectures and also use pass by reference, so we dont have to copy and define variables all the time.

The KCP-code is more work in progress, but we have been able to send the elevator object between several computers. I mention again it is work in progess and is not set up as it would be under testing, but it consists of two files; echo.go and elevio.go.
