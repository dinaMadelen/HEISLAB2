#!/bin/bash

# Kill any previously running instances
pkill -f "SimElevatorServer" || true
pkill -f "./main" || true
sleep 1

# Define ports and IDs for the three elevators
PORTS=("15657" "15658" "15659")
IDS=("127.0.0.1" "127.0.0.1" "127.0.0.1")
BROADCAST_PORTS=("30003" "30003" "30003") # Same broadcast port for all to enable communication

# Get the directory where the script is located
DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"

# Function to start a simulator instance in a new terminal
start_simulator() {
    local port=$1
    
    # Choose the right terminal command based on OS
    if [[ "$OSTYPE" == "linux-gnu"* ]]; then
        # Linux (assuming gnome-terminal)
        gnome-terminal -- bash -c "cd $DIR/build && ./SimElevatorServer --port $port; exec bash" &
    elif [[ "$OSTYPE" == "darwin"* ]]; then
        # macOS
        osascript -e "tell app \"Terminal\" to do script \"cd $DIR/build && ./SimElevatorServer --port $port\"" &
    else
        # Fallback for other systems
        echo "Starting simulator on port=$port in the background..."
        cd $DIR/build && ./SimElevatorServer --port $port &
    fi
    
    # Give the simulator time to start up
    sleep 2
}

# Function to start a main program instance in a new terminal
start_main() {
    local port=$1
    local id=$2
    local broadcast_port=$3
    
    # Choose the right terminal command based on OS
    if [[ "$OSTYPE" == "linux-gnu"* ]]; then
        # Linux (assuming gnome-terminal)
        gnome-terminal -- bash -c "cd $DIR/build && ./main --port=$port --id=$id --broadcast=$broadcast_port; exec bash" &
    elif [[ "$OSTYPE" == "darwin"* ]]; then
        # macOS
        osascript -e "tell app \"Terminal\" to do script \"cd $DIR/build && ./main --port=$port --id=$id --broadcast=$broadcast_port\"" &
    else
        # Fallback for other systems
        echo "Starting main program with port=$port, id=$id in the background..."
        cd $DIR/build && ./main --port=$port --id=$id --broadcast=$broadcast_port &
    fi
}

# Start all three simulators and elevator instances
echo "Starting 3 SimElevatorServer instances and 3 main program instances..."

for i in {0..2}; do
    echo "Starting set ${i+1}: SimElevatorServer on port=${PORTS[$i]}"
    start_simulator "${PORTS[$i]}"
    
    echo "Starting set ${i+1}: main program with port=${PORTS[$i]}, id=${IDS[$i]}"
    start_main "${PORTS[$i]}" "${IDS[$i]}" "${BROADCAST_PORTS[$i]}"
    
    # Short delay between starting sets
    sleep 1
done

echo "All simulators and elevator programs started."
echo "Set 1: port=${PORTS[0]}, id=${IDS[0]}"
echo "Set 2: port=${PORTS[1]}, id=${IDS[1]}"
echo "Set 3: port=${PORTS[2]}, id=${IDS[2]}"
echo ""
echo "To stop all instances, you can run: pkill -f 'SimElevatorServer' && pkill -f './main'"
