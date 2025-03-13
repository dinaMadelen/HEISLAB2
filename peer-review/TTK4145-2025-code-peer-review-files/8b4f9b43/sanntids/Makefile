# Variables
SIMULATOR_DIR := lib/Simulator-v2
SIMULATOR_BINARY := build/SimElevatorServer
COST_FN_DIR := lib/Project-resources/cost_fns/hall_request_assigner
COST_FN_BINARY := build/hall_request_assigner
GO_BINARY := build/main

# Default target
all: clean build_dirs $(SIMULATOR_BINARY) $(GO_BINARY) $(COST_FN_BINARY)

# Create build directories if they don't exist
build_dirs:
	mkdir -p build

# Compile the D simulator manually
$(SIMULATOR_BINARY):
	dmd -w -g $(SIMULATOR_DIR)/src/sim_server.d $(SIMULATOR_DIR)/src/timer_event.d -of$(SIMULATOR_BINARY)
	cp $(SIMULATOR_DIR)/simulator.con build/

# Compile the D cost func manually
$(COST_FN_BINARY):
	dmd -w -g $(COST_FN_DIR)/main.d \
		$(COST_FN_DIR)/config.d \
		$(COST_FN_DIR)/elevator_algorithm.d \
		$(COST_FN_DIR)/elevator_state.d \
		$(COST_FN_DIR)/optimal_hall_requests.d \
		$(COST_FN_DIR)/d-json/jsonx.d \
		-of$(COST_FN_BINARY)

# Build the Go application
$(GO_BINARY): $(wildcard cmd/*.go pkg/**/*.go)
	go build -o $(GO_BINARY) ./cmd

# Clean up build files
clean:
	rm -rf build

# Update submodules
submodule:
	git submodule update --init --recursive

.PHONY: all build_dirs clean submodule
