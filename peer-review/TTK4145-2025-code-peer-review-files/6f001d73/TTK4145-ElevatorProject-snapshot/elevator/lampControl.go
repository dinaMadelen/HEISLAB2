package elevator

import (
	"Driver-go/config"
	"Driver-go/elevio"
)

func SetPanelLamps(orders Orders, state State) {
	// Update hall&cab button lights
	for f := 0; f < config.NumFloors; f++ {
		elevio.SetButtonLamp(elevio.BT_Cab, f, orders[f][elevio.BT_Cab])
		for b := 0; b < config.NumButtons-1; b++ {
			elevio.SetButtonLamp(elevio.ButtonType(b), f, orders[f][b])
		}
	}
	// Update floor indicator and stop light
	elevio.SetFloorIndicator(state.Floor)
	elevio.SetStopLamp(state.Motorstop)
}

func SetDoorLamp(open bool) {
	elevio.SetDoorOpenLamp(open)
}
