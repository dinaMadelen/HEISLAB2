package elevator

import (
	"Driver-go/config"
	"Driver-go/elevio"
	"time"
)

type DoorState int

const (
	Closed      DoorState = 0
	InCountDown DoorState = 1
	Obstructed  DoorState = 2
)

func ManageDoor(
	doorOpenC <-chan bool,
	doorClosedC chan<- bool,
	doorObstructedC chan<- bool,
) {
	doorState := Closed
	obstruction := false

	obstructionC := make(chan bool)
	go elevio.PollObstructionSwitch(obstructionC)

	timeCounter := time.NewTimer(time.Hour)
	defer timeCounter.Stop()

	SetDoorLamp(false)

	for {
		select {
		// Handle obstruction detection
		case obstruction = <-obstructionC:
			if doorState == Obstructed && !obstruction {
				SetDoorLamp(false)
				doorClosedC <- true
				doorState = Closed
			}
			select {
			case doorObstructedC <- obstruction:
			default:
			}

		// Handle door opening request
		case <-doorOpenC:
			if doorState == Closed {
				SetDoorLamp(true)
				timeCounter.Reset(config.DoorOpenDuration)
				doorState = InCountDown
			} else if doorState == InCountDown || doorState == Obstructed {
				timeCounter.Reset(config.DoorOpenDuration)
				doorState = InCountDown
			} else {
				panic("Door state not implemented")
			}

		// Handle door closing after countdown
		case <-timeCounter.C:
			if doorState != InCountDown {
				panic("Unexpected timer event in wrong state")
			}
			if obstruction {
				doorState = Obstructed
			} else {
				SetDoorLamp(false)
				doorClosedC <- true
				doorState = Closed
			}
		}
	}
}
