package worldview

import (
	"TTK4145-Heislab/AssignerExecutable"
	"TTK4145-Heislab/configuration"
	"TTK4145-Heislab/driver-go/elevio"
	"TTK4145-Heislab/single_elevator"
)


func InitializeWorldView(elevatorID string) WorldView {
	message := WorldView{
		ID:                 elevatorID,
		Acklist:            make([]string, 0),
		ElevatorStatusList: map[string]single_elevator.State{elevatorID: single_elevator.State},
		HallOrderStatus:    InitializeHallOrderStatus(),
	}
	return message
}

func InitializeHallOrderStatus() [][configuration.NumButtons - 1]configuration.RequestState {
	HallOrderStatus := make([][configuration.NumButtons - 1]configuration.RequestState, configuration.NumFloors)
	for floor := range HallOrderStatus {
		for button := range HallOrderStatus[floor] {
			HallOrderStatus[floor][button] = configuration.None
		}
	}
	return HallOrderStatus
}


func updateWorldViewWithButton(localWorldView *WorldView, buttonPressed elevio.ButtonEvent, B bool) WorldView {
	if B == true { //mottar knappetrykk som ny bestilling (buttonpressedchannel)
		if buttonPressed == elevio.BT_HallDown || elevio.BT_HallUp {
			localWorldView.HallOrderStatus[buttonPressed.Floor][buttonPressed.Button] = configuration.Unconfirmed 
		} 
		if buttonPressed == elevio.BT_Cab { //her må worldview være local
			localWorldView.CabRequests[buttonPressed.Floor] = true 
		}
	} else { //sender tilbake knappetrykk fra FSM (ordercompletedchannel)
		if buttonPressed == elevio.BT_HallDown || elevio.BT_HallUp {
			localWorldView.HallOrderStatus[buttonPressed.Floor][buttonPressed.Button] = configuration.Completed
		} 
		if buttonPressed == elevio.BT_Cab { //her må worldview være local
			localWorldView.CabRequests[buttonPressed.Floor] = false 
		}
	}
	return localWorldView
}


func ResetAckList(localWorldView *WorldView) {
	localWorldView.Acklist = make([]string, 0)
	localWorldView.Acklist = append(localWorldView.Acklist, localWorldView.ID)
}

func ConvertHallOrderStatustoBool(WorldView WorldView) [][2]bool {
	boolMatrix := make([][2]bool, configuration.NumFloors)
	for floor := 0; floor < configuration.NumFloors; floor++ {
		for button := 0; button < 2; button++ {
			if WorldView.HallOrderStatus[floor][button] == configuration.Confirmed {
				boolMatrix[floor][button] = true
			} else {
				boolMatrix[floor][button] = false
			}
		}
	}
	return boolMatrix
}

//denne er ikke riktig og må gjøres på nytt - tanken er at vi skal formattere worldview til HRAInput 
func HRAInputFormatting(WorldView WorldView) AssignerExecutable.HRAInput {
	elevatorStates := make(map[string]AssignerExecutable.HRAElevState)
	hallrequests := ConvertHallOrderStatustoBool(WorldView)

	for ID := range WorldView.Acklist {
		if !WorldView.ElevatorStatusList[WorldView.Acklist[ID]].Unavailable { //har ikke en unavailable
			elevatorStates[WorldView.Acklist[ID]] = AssignerExecutable.HRAElevState{
				Behaviour: single_elevator.ToString(WorldView.ElevatorStatusList[WorldView.Acklist[ID]].Behaviour),
				Floor:     WorldView.ElevatorStatusList[WorldView.Acklist[ID]].Floor,
				Direction: elevio.DirToString(elevio.MotorDirection(WorldView.ElevatorStatusList[WorldView.Acklist[ID]].Direction)), //Direction: elevio.DirToString(WorldView.ElevatorStatusList[WorldView.Acklist[ID]].Direction),
				//CABREQUESTS - hvordan håndtere (HARAINput har CAB requests)
			}
		}
	}
	input := AssignerExecutable.HRAInput{
		HallRequests: hallrequests,
		States:       elevatorStates,
	}
	return input
}

func MergeCABandHRAout(OurHall [][2]bool, Ourcab []bool) single_elevator.Orders {
	var OrderMatrix single_elevator.Orders 
	for floor, cabbutton := range Ourcab {
		if cabbutton { 
			OrderMatrix[floor][2] = true 
		}
	}
	//ikke riktig iterasjon??
	for floor, buttons := range OurHall { 
		for buttonType, isPressed := range buttons {
			if isPressed { 
				OrderMatrix[floor][buttonType] = true 
			}
		}
	}
	return OrderMatrix
}


func AssignOrder(WorldView WorldView) map[string][][2]bool { 
	input := HRAInputFormatting(WorldView) 
	outputAssigner := AssignerExecutable.Assigner(input)
	return outputAssigner 

}


//denne er ikke ferdig og er egt hele heisen sin logikk 
	func MergeWorldViews(localWorldView WorldView, updatedWorldView WorldView, IDsAliveElevators  []string) WorldView {

		if len(localWorldView.Acklist) < len(updatedWorldView.Acklist) {
			localWorldView = &updatedWorldView
			localWorldView.Acklist = append(localWorldView.ID)
		}
		if len(localWorldView.Acklist) = len(updatedWorldView.Acklist) {
			continue 
		} 
	}
//alle må ha oppdatert worldview før den kan assignes og utføres 



func GetOurCAB(localWorldView WorldView, ourID string) []bool {
	return localWorldView.ElevatorStatusList[ourID].Cab
}
