package lights

import (
	"Driver-go/elevio"
	. "Driver-go/types"
)


func SetHallLights(req [NUMFLOORS][NUMBUTTONTYPE]bool) [NUMFLOORS][NUMBUTTONTYPE]bool {
	for floor := 0; floor < NUMFLOORS; floor++ {
		for btn := 0; btn < NUMBUTTONTYPE-1; btn++ {
			state := req[floor][btn]
			elevio.SetButtonLamp(ButtonType(btn), floor, state)
		}
	}
	return req
}


func SetCabLights(req [NUMFLOORS][NUMBUTTONTYPE]bool) [NUMFLOORS][NUMBUTTONTYPE]bool {
	for floor := 0; floor < NUMFLOORS; floor++ {
		state := req[floor][2]
		elevio.SetButtonLamp(BT_Cab, floor, state)
	}
	return req
}
