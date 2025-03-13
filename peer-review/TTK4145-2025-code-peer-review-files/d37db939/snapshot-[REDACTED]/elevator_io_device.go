package main

// functions for reading the input devices of the elevator
type ElevInputDevice struct {
	floorSensor   func() int
	requestButton func(int, ButtonType) int
	stopButton    func() int
	obstruction   func() int
}

// functions for controlling the output devices of the elevator
type ElevOutputDevice struct {
	floorIndicator     func(int)
	requestButtonLight func(int, ButtonType, int)
	doorLight          func(int)
	stopButtonLight    func(int)
	motorDirection     func(MotorDirection)
}

func elevioGetInputDevice() ElevInputDevice {
	return ElevInputDevice{
		floorSensor:   hardwareGetFloorSensorSignal,
		requestButton: hardwareGetButtonSignal,
		stopButton:    hardwareGetStopSignal,
		obstruction:   hardwareGetObstructionSignal,
	}
}

func elevioGetOutputDevice() ElevOutputDevice {
	return ElevOutputDevice{
		floorIndicator:     func(floor int) { hardwareLampToggle(LAMP_FLOOR_INDICATOR, 0, floor, 0) },
		requestButtonLight: func(floor int, button ButtonType, value int) { hardwareLampToggle(LAMP_BUTTON, button, floor, value) },
		doorLight:          func(value int) { hardwareLampToggle(LAMP_DOOR, 0, 0, value) },
		stopButtonLight:    func(value int) { hardwareLampToggle(LAMP_STOP, 0, 0, value) },
		motorDirection:     hardwareSetMotorDirection,
	}
}