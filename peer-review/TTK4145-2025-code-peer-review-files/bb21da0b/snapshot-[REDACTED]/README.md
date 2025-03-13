# TTK4145-Project
The system contains one master and two slaves. The slaves forwards commands from the master to its elevator. UDP is used. Hall down and Hall up button lights, floor lights and floor indicator are not implemented yet. We use C because it is used the most for embedded systems in the industry.

# Build
This project supports linux with gcc only. Open a terminal in the project root and type the following:
```
mkdir build
cd build
cmake ..
make
```
This will generate two executables, one for master and the other for slave.