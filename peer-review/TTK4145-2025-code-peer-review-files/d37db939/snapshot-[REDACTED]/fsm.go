package main

var (
	elevator     Elevator
	outputDevice ElevOutputDevice
)

// initializing the elevator state and output device
func init() {
	elevator = elevatorUninitialized()
	// Simulating config loading
	elevator.config.doorOpenDurationS = 3.0 // Example default value
	elevator.config.clearRequests = CLEAR_ALL
	outputDevice = elevioGetOutputDevice()
}

// updates all request button lights based on the request state of the elevator
func setAllLights(e Elevator) {
	for floor := 0; floor < N_FLOORS; floor++ {
		for btn := 0; btn < N_BUTTONS; btn++ {
			outputDevice.requestButtonLight(floor, ButtonType(btn), boolToInt(e.requests[floor][btn]))
		}
	}
}

// moving the elevator downwards to find a valid floor when it is initialized between floors
func fsmOnInitBetweenFloors() {
	outputDevice.motorDirection(DIRN_DOWN)
	elevator.dirn = DIRN_DOWN
	elevator.state = ELEVSTATE_MOVING
}

// when a request button is pressed - updates elevator state and processed the request based on the current state
func fsmOnRequestButtonPress(btnFloor int, btnType ButtonType) {
	// fmt.Printf("\n\nfsmOnRequestButtonPress(%d, %s)\n", btnFloor, elevioButtonToString(btnType))

	switch elevator.state {
	case ELEVSTATE_DOOR_OPEN:
		if requestsShouldClearImmediately(elevator, btnFloor, btnType) {
			timerStart(elevator.config.doorOpenDurationS)
		} else {
			elevator.requests[btnFloor][btnType] = true
		}
	case ELEVSTATE_MOVING:
		elevator.requests[btnFloor][btnType] = true
	case ELEVSTATE_IDLE:
		elevator.requests[btnFloor][btnType] = true
		pair := requestsChooseDirection(elevator)
		elevator.dirn = pair.dirn
		elevator.state = pair.state
		switch pair.state {
		case ELEVSTATE_DOOR_OPEN:
			outputDevice.doorLight(1)
			timerStart(elevator.config.doorOpenDurationS)
			elevator = requestsClearAtCurrentFloor(elevator)
		case ELEVSTATE_MOVING:
			outputDevice.motorDirection(elevator.dirn)
		}
	}

	setAllLights(elevator)
}

// when the elevator arrives at a floor - updates the floor indicator, stops the elevator if needed, processes requests
func fsmOnFloorArrival(newFloor int) {
	// fmt.Printf("\n\nfsmOnFloorArrival(%d)\n", newFloor)
	elevator.floor = newFloor
	outputDevice.floorIndicator(elevator.floor)

	if elevator.state == ELEVSTATE_MOVING && requestsShouldStop(elevator) {
		outputDevice.motorDirection(DIRN_STOP)
		outputDevice.doorLight(1)
		elevator = requestsClearAtCurrentFloor(elevator)
		timerStart(elevator.config.doorOpenDurationS)
		setAllLights(elevator)
		elevator.state = ELEVSTATE_DOOR_OPEN
	}
}

// when the door timer expires - checks for obstruction and determines the next state of the elevator
func fsmOnDoorTimeout() {
	// fmt.Println("\n\nfsmOnDoorTimeout()")

	if hardwareGetObstructionSignal() != 0 {
		timerStart(elevator.config.doorOpenDurationS)
		return
	}

	if elevator.state == ELEVSTATE_DOOR_OPEN {
		pair := requestsChooseDirection(elevator)
		elevator.dirn = pair.dirn
		elevator.state = pair.state
		switch elevator.state {
		case ELEVSTATE_DOOR_OPEN:
			timerStart(elevator.config.doorOpenDurationS)
			elevator = requestsClearAtCurrentFloor(elevator)
			setAllLights(elevator)
		case ELEVSTATE_MOVING, ELEVSTATE_IDLE:
			outputDevice.doorLight(0)
			outputDevice.motorDirection(elevator.dirn)
		}
	}
}

// when an obstruction signal is detected - keeps door open and stops door timer
func fsmOnObstruction() {
	if elevator.state == ELEVSTATE_DOOR_OPEN {
		outputDevice.doorLight(1) // Keep door open
		timerStop()               // Stop the timer while obstructed
	}
}

// when an obstruction signal is cleared - restarts door timer if the door is open
func fsmOnObstructionCleared() {
	if elevator.state == ELEVSTATE_DOOR_OPEN {
		outputDevice.doorLight(1)                     // Keep door open
		timerStart(elevator.config.doorOpenDurationS) // Restart door timer
	}
}


func boolToInt(b bool) int {
	if b {
		return 1
	}
	return 0
}
