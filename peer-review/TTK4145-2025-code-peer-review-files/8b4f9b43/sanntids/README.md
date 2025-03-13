# Simulator MkII - Setup Guide
Test
## Prerequisites

Ensure the D Compiler and golang compiler is installed:

```bash
git clone --recurse-submodules git@github.com:hopedantilope/sanntids.git
cd sanntids
```

Build with
```bash

make
cd build/
```

To run simulator use 

```bash
./SimElevatorServer --port <port> --numfloors <num-floors>
```
To run main 
```bash
./main
```
