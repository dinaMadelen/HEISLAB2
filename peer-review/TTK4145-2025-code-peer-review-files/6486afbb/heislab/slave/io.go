package slave

import (
	"github.com/Kirlu3/Sanntid-G30/heislab/config"
	"github.com/Kirlu3/Sanntid-G30/heislab/driver-go/elevio"
)

const doorOpenDuration = 3
const timeBetweenFloors = 5

/*
	Activates after a new elevator object is created
	Interfaces with the elevator hardware to update the lights and motor direction
	If the door opens or the elevator starts moving, the corresponding timer is started

Input: The new elevator object, the old elevator object, and the start time channel
*/
func io_activateIO(n_elevator Elevator, t_start chan int) {

	elevio.SetFloorIndicator(n_elevator.Floor) //Floor IO

	switch n_elevator.Behaviour {
	case EB_DoorOpen:
		t_start <- doorOpenDuration
		elevio.SetDoorOpenLamp(true)
		elevio.SetMotorDirection(elevio.MD_Stop)
	case EB_Moving:
		t_start <- timeBetweenFloors
		elevio.SetDoorOpenLamp(false)
		elevio.SetMotorDirection(elevio.MotorDirection(n_elevator.Direction))
	case EB_Idle:
		elevio.SetDoorOpenLamp(false)
		elevio.SetMotorDirection(elevio.MD_Stop)
	}
}

/*
	Updates the lights on the elevator panel
	Interfaces with the elevator hardware to update the lights

Input: Array of lights to be turned on or off
*/
func io_updateLights(lights [config.N_FLOORS][config.N_BUTTONS]bool) {
	for i := range config.N_FLOORS {
		for j := range config.N_BUTTONS {
			elevio.SetButtonLamp(elevio.ButtonType(j), i, lights[i][j])
		}
	}

}
